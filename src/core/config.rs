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
        let encrypted_password = generate_encrypted_password_from_gui(&gui_config);
        
        AccountConfig {
            username: gui_config.username.clone(),
            encrypted_password,
            isp: normalize_isp(&gui_config.isp),
        }
    }
}

/// 生成加密密码的统一函数
fn generate_encrypted_password_from_gui(gui_config: &GuiConfigDto) -> String {
    if !gui_config.password.is_empty() {
        generate_encrypted_password(&gui_config.password)
    } else {
        // 如果没有提供新密码，使用已有的加密密码
        gui_config.encrypted_password.clone()
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
        if let Some(last_time) = *last_save_time {
            if now.duration_since(last_time) < self.debounce_delay {
                return Ok(());
            }
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
    
    let config: ConfigData = toml::from_str(&content)
        .map_err(|e| AppError::ConfigError(format!("配置文件格式错误: {}", e)))?;
    
    Ok(config)
}

/// 保存配置
pub fn save_config(config: &ConfigData) -> AppResult<()> {
    let config_path = get_config_path();
    
    if let Some(parent) = Path::new(&config_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::ConfigError(format!("无法创建配置目录: {}", e)))?;
    }
    
    let content = toml::to_string_pretty(config)
        .map_err(|e| AppError::ConfigError(format!("无法序列化配置: {}", e)))?;
    
    fs::write(&config_path, content)
        .map_err(|e| AppError::ConfigError(format!("无法写入配置文件 '{}': {}", config_path, e)))?;
    
    Ok(())
}

/// 检查配置是否完整
pub fn is_config_complete(config: &ConfigData) -> bool {
    !config.account.username.is_empty() && 
    !config.account.encrypted_password.is_empty()
}

lazy_static! {
    static ref CONFIG: Mutex<ConfigData> = Mutex::new(load_config().unwrap_or_default());
}

pub fn get_global_config() -> std::sync::MutexGuard<'static, ConfigData> {
    CONFIG.lock().unwrap()
}