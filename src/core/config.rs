//! 配置管理模块
//!
//! 负责处理应用程序的所有配置相关功能

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use toml;

use crate::core::error::{AppError, AppResult};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex as TokioMutex;

use crate::core::crypto::generate_encrypted_password;
use crate::core::dto::GuiConfigDto;
use crate::core::network::NetworkConfig;

const DEFAULT_LOG_FILE_PATH: &str = "./AutoLogin.log";
const CONFIG_FILE_NAME: &str = "config.toml";

/// 账户配置信息
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct AccountConfig {
    /// 用户名
    pub username: String,
    /// 加密后的密码
    pub encrypted_password: String,
    /// ISP运营商
    pub isp: String,
}

impl From<GuiConfigDto> for AccountConfig {
    fn from(gui_config: GuiConfigDto) -> Self {
        let encrypted_password = if !gui_config.password.is_empty() {
            // 只有在提供了新密码时才进行加密
            generate_encrypted_password(&gui_config.password)
        } else {
            // 否则使用已有的加密密码
            gui_config.encrypted_password.clone()
        };

        AccountConfig {
            username: gui_config.username.clone(),
            encrypted_password,
            isp: normalize_isp(&gui_config.isp),
        }
    }
}

/// 标准化ISP值的统一函数
pub fn normalize_isp(isp: &str) -> String {
    if isp == "校园网" {
        String::new()
    } else {
        isp.to_string()
    }
}

/// 日志配置信息
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LoggingConfig {
    /// 是否启用日志记录
    pub enable_logging: bool,
    /// 日志文件路径
    pub log_file_path: String,
    /// INFO等级的日志保留天数
    pub info_log_retention_days: i64,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            enable_logging: true,
            log_file_path: DEFAULT_LOG_FILE_PATH.to_string(),
            info_log_retention_days: 7,
        }
    }
}

/// 消息配置信息
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct MessageConfig {
    /// 系统通知文本模板
    pub notify_text: String,
    /// GUI消息框文本模板
    pub gui_text: String,
    /// 日志记录文本模板
    pub log_text: String,
}

impl Default for MessageConfig {
    fn default() -> Self {
        // 默认使用校园网运营商的配置（包含流量信息%4）
        MessageConfig::for_campus_network()
    }
}

impl MessageConfig {
    /// 为校园网运营商创建默认消息配置（包含流量信息%4）
    pub fn for_campus_network() -> Self {
        MessageConfig {
            notify_text: "%1 %2\n%3 %4".to_string(),
            gui_text: "%1 %2".to_string(),
            log_text: "%1 %2 %3 %4".to_string(),
        }
    }

    /// 为非校园网运营商创建默认消息配置（不包含流量信息%4）
    pub fn for_non_campus_network() -> Self {
        MessageConfig {
            notify_text: "%1 %2 %3".to_string(),
            gui_text: "%1 %2".to_string(),
            log_text: "%1 %2 %3".to_string(),
        }
    }
}

/// 设置配置信息
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct SettingsConfig {
    /// 是否开机自启
    #[serde(default)]
    pub auto_start: bool,
}

/// 完整配置数据结构
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct ConfigData {
    /// 账户配置
    #[serde(default)]
    pub account: AccountConfig,
    /// 网络配置
    #[serde(default)]
    pub network: NetworkConfig,
    /// 日志配置
    #[serde(default)]
    pub logging: LoggingConfig,
    /// 设置配置
    #[serde(default)]
    pub settings: SettingsConfig,
    /// 消息配置
    #[serde(default)]
    pub message: MessageConfig,
}

impl ConfigData {
    /// 加载现有配置或返回默认配置
    pub fn load_existing_or_default() -> Self {
        load_config().unwrap_or_default()
    }
}

/// 配置管理器，用于管理配置的保存操作
pub struct ConfigManager {
    last_save_time: Arc<TokioMutex<Option<Instant>>>,
    debounce_delay: Duration,
}

impl ConfigManager {
    /// 创建新的配置管理器实例
    pub fn new() -> Self {
        Self {
            last_save_time: Arc::new(TokioMutex::new(None)),
            debounce_delay: Duration::from_millis(500), // 500ms 防抖延迟
        }
    }

    /// 带防抖功能的保存配置
    pub async fn save_config_with_debounce(&self, config: ConfigData) -> AppResult<()> {
        let mut last_save_time = self.last_save_time.lock().await;

        let now = Instant::now();
        if let Some(last_time) = *last_save_time
            && now.duration_since(last_time) < self.debounce_delay
        {
            return Ok(());
        }

        *last_save_time = Some(now);

        save_config(&config)
    }

    /// 立即保存配置
    pub fn save_config_immediately(&self, config: &ConfigData) -> AppResult<()> {
        save_config(config)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取配置文件路径
pub fn get_config_path() -> String {
    CONFIG_FILE_NAME.to_string()
}

/// 加载配置
pub fn load_config() -> AppResult<ConfigData> {
    let config_path = get_config_path();

    if !Path::new(&config_path).exists() {
        let default_config = ConfigData::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| AppError::ConfigError(format!("无法读取配置文件 '{}': {}", config_path, e)))?;

    let mut config: ConfigData = toml::from_str(&content)
        .map_err(|e| AppError::ConfigError(format!("配置文件格式错误: {}", e)))?;

    let mut config_modified = false;

    // 如果配置文件中没有message节，追加默认配置
    if !content.contains("[message]") {
        // 根据运营商类型设置不同的默认消息配置
        config.message = if config.account.isp.is_empty() {
            // 校园网运营商（isp为空）
            MessageConfig::for_campus_network()
        } else {
            // 非校园网运营商
            MessageConfig::for_non_campus_network()
        };
        config_modified = true;
    } else {
        // 处理转义的换行符：将 \n 转换回实际的换行符
        config.message.notify_text = config.message.notify_text.replace("\\n", "\n");
        config.message.gui_text = config.message.gui_text.replace("\\n", "\n");
        config.message.log_text = config.message.log_text.replace("\\n", "\n");
    }

    // 验证并同步开机自启配置
    #[cfg(windows)]
    {
        if validate_and_sync_auto_start_config(&mut config) {
            config_modified = true;
        }
    }

    // 如果修改了配置（添加了消息配置节或同步了开机自启配置），则保存配置
    if config_modified {
        save_config(&config)?;
    }

    Ok(config)
}

/// 保存配置
pub fn save_config(config: &ConfigData) -> AppResult<()> {
    let config_path = get_config_path();

    if let Some(parent) = Path::new(&config_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::ConfigError(format!("无法创建配置目录: {}", e)))?;
    }

    let mut content = String::new();

    // 账户配置
    content.push_str("[account]\n");
    content.push_str(&format!("username = \"{}\"\n", config.account.username));
    content.push_str(&format!(
        "encrypted_password = \"{}\"\n",
        config.account.encrypted_password
    ));
    content.push_str(&format!("isp = \"{}\"\n\n", config.account.isp));

    // 网络配置
    content.push_str("[network]\n");
    content.push_str(&format!("login_ip = \"{}\"\n", config.network.login_ip));
    content.push_str(&format!(
        "result_return = \'{}\'\n",
        config.network.result_return
    ));
    content.push_str(&format!(
        "signed_in_title = \"{}\"\n",
        config.network.signed_in_title
    ));
    content.push_str(&format!(
        "not_sign_in_title = \"{}\"\n\n",
        config.network.not_sign_in_title
    ));

    // 日志配置
    content.push_str("[logging]\n");
    content.push_str(&format!(
        "enable_logging = {}\n",
        config.logging.enable_logging
    ));
    content.push_str(&format!(
        "log_file_path = \"{}\"\n",
        config.logging.log_file_path
    ));
    content.push_str(&format!(
        "info_log_retention_days = {}\n\n",
        config.logging.info_log_retention_days
    ));

    content.push_str("[settings]\n");
    content.push_str(&format!("auto_start = {}\n\n", config.settings.auto_start));

    content.push_str("[message]\n");
    content.push_str(&format!(
        "notify_text = \"{}\"\n",
        config.message.notify_text.replace("\n", "\\n")
    ));
    content.push_str(&format!(
        "gui_text = \"{}\"\n",
        config.message.gui_text.replace("\n", "\\n")
    ));
    content.push_str(&format!(
        "log_text = \"{}\"\n",
        config.message.log_text.replace("\n", "\\n")
    ));

    fs::write(&config_path, content)
        .map_err(|e| AppError::ConfigError(format!("无法写入配置文件 '{}': {}", config_path, e)))?;

    Ok(())
}

/// 检查配置是否完整
pub fn is_config_complete(config: &ConfigData) -> bool {
    !config.account.username.is_empty() && !config.account.encrypted_password.is_empty()
}

/// 验证消息配置是否合法
pub fn validate_message_config(config: &MessageConfig) -> bool {
    let templates = [&config.notify_text, &config.gui_text, &config.log_text];

    // 检查所有模板中是否至少有一个包含占位符
    let mut has_placeholder = false;

    for template in templates {
        if template.contains("%1")
            || template.contains("%2")
            || template.contains("%3")
            || template.contains("%4")
        {
            has_placeholder = true;
            break;
        }
    }

    has_placeholder
}

lazy_static! {
    static ref CONFIG: Mutex<ConfigData> = Mutex::new(load_config().unwrap_or_default());
}

pub fn get_global_config() -> std::sync::MutexGuard<'static, ConfigData> {
    CONFIG.lock().unwrap()
}

/// 检查注册表中开机自启项是否存在
#[cfg(windows)]
pub fn is_auto_start_registry_exists() -> bool {
    use std::env;
    use winreg::RegKey;
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (reg_path, app_name) = (
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        "AutoLoginGuet",
    );

    if let Ok(reg_key) = hkcu.open_subkey_with_flags(reg_path, KEY_READ)
        && let Ok(value) = reg_key.get_value::<String, _>(app_name)
        && let Ok(exe_path) = env::current_exe() {
            let expected_value =
            format!("\"{}\" -silent", exe_path.to_str().unwrap_or_default());
            return value == expected_value;
        }
    false
}

/// 验证并同步开机自启配置，与注册表状态一致
#[cfg(windows)]
pub fn validate_and_sync_auto_start_config(config: &mut ConfigData) -> bool {
    let registry_exists = is_auto_start_registry_exists();

    // 如果配置与注册表状态不一致，需要同步
    if config.settings.auto_start != registry_exists {
        config.settings.auto_start = registry_exists;
        // 返回true表示配置已更改
        true
    } else {
        // 配置一致，无需更改
        false
    }
}
