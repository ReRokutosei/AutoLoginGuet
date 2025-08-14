use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use toml;
use std::env;
use base64::{Engine as _, engine::general_purpose};

use crate::core::network::NetworkConfig;

const DEFAULT_LOG_FILE_PATH: &str = "./AutoLogin.log";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct AccountConfig {
    pub username: String,
    pub encrypted_password: String,
    pub isp: String,
}

impl Default for AccountConfig {
    fn default() -> Self {
        AccountConfig {
            username: String::new(),
            encrypted_password: String::new(),
            isp: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LoggingConfig {
    pub enable_logging: bool,
    pub log_file_path: String,
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SettingsConfig {
    #[serde(default)]
    pub auto_start: bool,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        SettingsConfig {
            auto_start: false,
        }
    }
}

/// 配置结构体定义
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct ConfigData {
    pub network: NetworkConfig,
    pub notification: NotificationConfig,
    pub logging: LoggingConfig,
    pub account: AccountConfig,
    pub settings: SettingsConfig,
}

impl Default for ConfigData {
    fn default() -> Self {
        ConfigData {
            network: NetworkConfig::default(),
            notification: NotificationConfig::default(),
            logging: LoggingConfig::default(),
            account: AccountConfig::default(),
            settings: SettingsConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(default)]
pub struct NotificationConfig {}

impl Default for NotificationConfig {
    fn default() -> Self {
        NotificationConfig {}
    }
}

/// 配置文件操作函数
pub async fn load_config() -> Result<ConfigData, String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe_path.parent().ok_or("无法获取可执行文件目录")?;
    let config_path = exe_dir.join(CONFIG_FILE_NAME);
    
    if !config_path.exists() {
        let default_config = ConfigData::default();
        save_config(&default_config).map_err(|e| format!("保存默认配置失败: {}", e))?;
    }
    
    let contents = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut config: ConfigData = toml::from_str(&contents).map_err(|e| e.to_string())?;
    
    if config.logging.enable_logging {
        let log_path = Path::new(&config.logging.log_file_path);
        if let Some(log_path_str) = log_path.to_str() {
            if log_path_str.starts_with("./") || log_path_str.starts_with("../") {
                if let Ok(stripped_path) = log_path.strip_prefix("./") {
                    config.logging.log_file_path = exe_dir.join(stripped_path).to_string_lossy().to_string();
                } else if let Ok(stripped_path) = log_path.strip_prefix("../") {
                    config.logging.log_file_path = exe_dir.join(stripped_path).to_string_lossy().to_string();
                }
            }
        } else {
            return Err("日志路径格式错误".to_string());
        }
    }
    
    Ok(config)
}

pub fn save_config(config: &ConfigData) -> Result<(), String> {
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe_path.parent().ok_or("无法获取可执行文件目录")?;
    let config_path = exe_dir.join(CONFIG_FILE_NAME);
    
    let toml_content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(config_path, toml_content).map_err(|e| e.to_string())
}

pub fn is_config_complete(config: &ConfigData) -> bool {
    if config.account.username.is_empty() {
        eprintln!("错误：用户名不能为空");
        return false;
    }
    
    if config.account.encrypted_password.is_empty() {
        eprintln!("错误：密码不能为空");
        return false;
    }
    
    if !config.account.username.chars().all(|c| c.is_ascii_digit()) {
        eprintln!("错误：学号格式不正确");
        return false;
    }
    
    if general_purpose::STANDARD.decode(&config.account.encrypted_password).is_err() {
        eprintln!("错误：密码格式不正确");
        return false;
    }
    
    match config.account.isp.as_str() {
        "" | "校园网" | "@cmcc" | "@unicom" | "@telecom" => true,
        _ => {
            eprintln!("错误：无效的运营商选择 - {}", config.account.isp);
            false
        }
    }
}