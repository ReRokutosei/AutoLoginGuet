//! GUI状态管理

use autologinguet_core::core::config::ConfigData;
use autologinguet_core::core::dto::GuiConfigDto;
use dioxus::prelude::*;

/// GUI配置数据结构
#[derive(Clone, PartialEq, Default)]
pub struct GuiConfigWithData {
    /// GUI配置信息
    pub gui_config: GuiConfigDto,
    /// 加密后的密码
    pub encrypted_password: String,
    /// 完整配置数据
    pub full_config: ConfigData,
}

impl From<ConfigData> for GuiConfigWithData {
    /// 从ConfigData转换为GuiConfigWithData
    fn from(config: ConfigData) -> Self {
        let encrypted_password = config.account.encrypted_password.clone();
        GuiConfigWithData {
            gui_config: GuiConfigDto::from(config.clone()),
            encrypted_password,
            full_config: config,
        }
    }
}

/// GUI状态管理
#[derive(Default, Clone, PartialEq)]
pub struct GuiState {
    /// GUI配置信号
    pub gui_config: Signal<GuiConfigDto>,
    /// 带数据的GUI配置信号
    pub gui_config_with_data: Signal<GuiConfigWithData>,
    /// 消息信号
    pub message: Signal<String>,
    /// 日志信号
    pub logs: Signal<String>,
    /// 会话日志信号
    pub session_logs: Signal<String>,
}
