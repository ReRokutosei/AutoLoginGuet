//! GUI服务模块
//!
//! 负责处理GUI与核心服务层之间的交互

use crate::gui::gui_event::GuiEventHandler;
use crate::gui::state::GuiConfigWithData;
use autologinguet_core::core::config::{load_config, ConfigData};
use autologinguet_core::core::decrypt_password_with_machine_key;
use autologinguet_core::core::error::AppResult;
use autologinguet_core::core::events::GuiEventHandlerMessage;
use autologinguet_core::core::service::{validate_password, validate_username};
use autologinguet_core::{AuthService, GuiConfigDto, MessageCenter};
use dioxus::prelude::*;
use std::sync::mpsc::Receiver;

/// GUI登录结果处理器
pub struct GUILoginResultHandler;

impl GUILoginResultHandler {
    pub fn new() -> Self {
        Self
    }
    
    /// 处理登录结果
    /// 
    /// # 参数
    /// * `result` - 登录结果
    /// * `message` - GUI消息信号
    /// * `session_logs` - GUI会话日志信号
    pub fn handle(
        &self,
        result: autologinguet_core::core::service::LoginResult,
        message: &mut Signal<String>) {
        // 直接更新消息显示为登录结果（不包含时间信息）
        *message.write() = result.message.clone();
    }
}

/// 初始化配置
pub async fn init_config(mut gui_config: Signal<GuiConfigDto>, mut gui_config_with_data: Signal<GuiConfigWithData>) -> Option<(AuthService, Receiver<GuiEventHandlerMessage>)> {
    match load_config() {
        Ok(config) => {
            let gui_config_data = GuiConfigWithData::from(config.clone());
            *gui_config_with_data.write() = gui_config_data.clone();
            *gui_config.write() = gui_config_data.gui_config;
            
            let mut auth_service = AuthService::new(config);
            let (event_handler, receiver) = GuiEventHandler::new();
            auth_service.set_event_handler(Box::new(event_handler));
            
            Some((auth_service, receiver))
        }
        Err(_) => {
            None
        }
    }
}

/// 初始化日志和网络状态
pub async fn init_logs_and_network(
    auth_service: &AuthService,
    mut message: Signal<String>,
    mut logs: Signal<String>
) {
    if let Ok(config) = load_config() {
        let message_center = MessageCenter::new(Some(config.logging.clone()), Default::default());
        if let Ok(log_content) = message_center.read_logs() {
            *logs.write() = log_content;
        }
    }

    if let Ok(status) = auth_service.check_network_status(false).await {
        let message_center = MessageCenter::default();
        let status_message = message_center.format_message(&status.to_message(), None);
        *message.write() = status_message;
    } else {
        message.set("网络检查失败".to_string());
    }
}

/// 保存账户配置
pub async fn save_account_config(auth_service: &AuthService, gui_config: &GuiConfigDto) -> AppResult<()> {
    let config_to_save: ConfigData = gui_config.clone().into();
    auth_service.save_config(&config_to_save)
}

/// 执行登录操作
pub async fn perform_login(
    auth_service: &AuthService,
    gui_config: &GuiConfigDto,
    gui_config_with_data: &GuiConfigWithData,
    mut message: Signal<String>) -> AppResult<()> {
    let message_center = MessageCenter::default();
    
    // 使用通用验证函数验证学号
    if !validate_username(&gui_config.username) && !gui_config.username.is_empty() {
        *message.write() = "账号格式不正确，请输入3-12位数字".to_string();
        return Err(autologinguet_core::core::error::AppError::ConfigError("账号格式不正确".to_string()));
    }
    
    // 使用通用验证函数验证密码（如果提供了新密码）
    if !gui_config.password.is_empty() && !validate_password(&gui_config.password) {
        *message.write() = "密码长度不正确，请输入8-32位密码".to_string();
        return Err(autologinguet_core::core::error::AppError::ConfigError("密码长度不正确".to_string()));
    }
    
    let _ = message_center.log_message("INFO", "正在尝试登录...", None);
    
    if !gui_config.username.is_empty() && !gui_config.password.is_empty() {
        match save_account_config(auth_service, gui_config).await {
            Ok(_) => {
                let _ = message_center.log_message("INFO", "账户信息已保存", None);
            }
            Err(e) => {
                let error_msg = format!("保存账户信息失败: {}", e);
                let _ = message_center.log_message("ERROR", &error_msg, None);
                *message.write() = format!("保存账户信息失败: {}", e);
                return Err(e);
            }
        }
    }

    let config_to_login = get_login_config(gui_config, gui_config_with_data);

    // 解密密码并执行登录
    let password = if !gui_config.password.is_empty() {
        gui_config.password.clone()
    } else {
        autologinguet_core::core::crypto::handle_password_decryption_error_with_default(
            decrypt_password_with_machine_key(&config_to_login.account.encrypted_password),
            auth_service.get_event_bus()
        )
    };

    let login_result = auth_service.login_with_credentials(
        &config_to_login.account.username,
        &password,
        &config_to_login.account.isp,
    ).await;

    handle_login_result(login_result, message).await
}

/// 统一处理登录结果
async fn handle_login_result(
    login_result: AppResult<autologinguet_core::core::service::LoginResult>,
    mut message: Signal<String>) -> AppResult<()> {
    match login_result {
        Ok(result) => {
            // 使用GUI登录结果处理器处理登录结果
            let handler = GUILoginResultHandler::new();
            handler.handle(result, &mut message);
            
            Ok(())
        }
        Err(e) => {
            *message.write() = format!("登录过程出错: {}", e);
            Err(e)
        }
    }
}

/// 获取登录配置
fn get_login_config(
    gui_config: &GuiConfigDto,
    gui_config_with_data: &GuiConfigWithData
) -> ConfigData {
    let mut config_to_login: ConfigData = gui_config.clone().into();
    
    // 只要密码为空且有已保存的加密密码，就使用已保存的密码
    if gui_config.password.is_empty() && !gui_config_with_data.encrypted_password.is_empty() {
        config_to_login.account.encrypted_password = gui_config_with_data.encrypted_password.clone();
    }
    
    config_to_login
}

/// 设置开机自启
#[cfg(windows)]
pub async fn set_auto_start(
    auth_service: &AuthService,
    enabled: bool,
    gui_config: &GuiConfigDto,
    gui_config_with_data: &GuiConfigWithData
) -> AppResult<()> {
    let mut config_to_save: ConfigData = gui_config.clone().into();
    config_to_save.settings.auto_start = enabled;
    
    if !gui_config.username.is_empty() && !gui_config.password.is_empty() {
    } else if !gui_config_with_data.encrypted_password.is_empty() {
        config_to_save.account.encrypted_password = gui_config_with_data.encrypted_password.clone();
    }
    
    auth_service.save_config(&config_to_save)?;
    auth_service.set_auto_start(enabled)
}