//! 数据传输对象(DTO)模块
//!
//! 用于GUI和核心模块之间数据传输的结构体

use crate::core::config::{AccountConfig, ConfigData, SettingsConfig};
use crate::core::normalize_isp;

/// GUI配置数据传输对象
#[derive(Clone, PartialEq, Default)]
pub struct GuiConfigDto {
    /// 用户名
    pub username: String,
    /// 密码（明文）
    pub password: String,
    /// 已加密的密码
    pub encrypted_password: String,
    /// 运营商
    pub isp: String,
    /// 是否开机自启
    pub auto_start: bool,
}

impl GuiConfigDto {
    /// 创建新的GUI配置DTO
    pub fn new(username: String, password: String, isp: String, auto_start: bool) -> Self {
        Self {
            username,
            password,
            encrypted_password: String::new(),
            isp,
            auto_start,
        }
    }
}

impl From<ConfigData> for GuiConfigDto {
    /// 从`ConfigData`转换为`GuiConfigDto`
    fn from(config: ConfigData) -> Self {
        Self {
            username: config.account.username,
            password: String::new(),
            encrypted_password: config.account.encrypted_password,
            isp: if config.account.isp.is_empty() {
                "校园网".to_string()
            } else {
                config.account.isp
            },
            auto_start: config.settings.auto_start,
        }
    }
}

impl From<GuiConfigDto> for ConfigData {
    /// 从`GuiConfigDto`转换为`ConfigData`
    fn from(gui_config: GuiConfigDto) -> Self {
        // 先加载现有配置
        let existing_config = ConfigData::load_existing_or_default();

        let encrypted_password = if !gui_config.password.is_empty() {
            // 只有在提供了新密码时才进行加密
            crate::core::crypto::generate_encrypted_password(&gui_config.password)
        } else {
            // 否则使用已有的加密密码
            gui_config.encrypted_password.clone()
        };

        ConfigData {
            network: existing_config.network,
            account: AccountConfig {
                username: gui_config.username.clone(),
                encrypted_password,
                isp: normalize_isp(&gui_config.isp),
            },
            logging: existing_config.logging,
            settings: SettingsConfig {
                auto_start: gui_config.auto_start,
            },
            message: existing_config.message,
        }
    }
}
