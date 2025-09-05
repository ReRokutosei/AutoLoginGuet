//! 调试模块
//!
//! 负责处理GUI界面中的调试模式功能，提供比正常模式更详细的信息输出

use autologinguet_core::core::config::{ConfigData, normalize_isp};
use autologinguet_core::core::dto::GuiConfigDto;
use autologinguet_core::core::events::EventBus;
use autologinguet_core::core::message::MessageCenter;
use autologinguet_core::core::network::NetworkManager;
use dioxus::prelude::*;

const DEFAULT_LOGIN_IP: &str = "http://10.0.1.5/";

/// Debug信息结构体
#[derive(Clone, PartialEq, Default)]
pub struct DebugInfo {
    pub enable_debug: bool,
    pub request_url: String,
    pub request_params: String,
    pub response_content: String,
    pub status_code: String,
    pub error_message: String,
}

/// 将`GuiConfigDto`转换为`ConfigData`用于Debug登录
fn gui_config_to_config(gui_config: &GuiConfigDto) -> ConfigData {
    let config: ConfigData = gui_config.clone().into();
    config
}

/// 执行Debug网络请求
pub fn perform_debug_network_request(
    gui_config: GuiConfigDto,
    mut debug_info: Signal<DebugInfo>,
    mut message: Signal<String>,
    mut session_logs: Signal<String>,
    current_log: String,
) {
    spawn(async move {
        let mut debug_output = current_log;
        let config_to_login: ConfigData = gui_config_to_config(&gui_config);
        let network_manager = NetworkManager::new(config_to_login.network.clone());
        let message_center = MessageCenter::new(None, EventBus::new());

        message_center.log_event("INFO", "开始调试网络请求...").ok();

        let password = autologinguet_core::core::crypto::decrypt_config_password(
            &config_to_login.account.encrypted_password,
        )
        .unwrap_or_else(|e| {
            message_center
                .log_event("ERROR", &format!("解密密码失败: {}", e))
                .ok();
            String::new()
        });

        let actual_isp = normalize_isp(&config_to_login.account.isp);

        debug_info.write().request_url = DEFAULT_LOGIN_IP.to_string();
        debug_info.write().request_params = format!(
            "callback=dr1003&DDDDD={}{}&upass=******&0MKKey=123456",
            config_to_login.account.username, actual_isp
        );

        debug_output.push_str(&format!("使用客户端配置: {:?}\n", config_to_login.network));

        let login_result = network_manager
            .attempt_login_with_credentials(
                &config_to_login.account.username,
                &password,
                &config_to_login.account.isp,
            )
            .await;

        match login_result {
            Ok(login_result) => {
                // 使用network.rs的LoginResult构建响应内容
                let response_content = format!(
                    "成功: {}, 校园网状态: {:?}, 耗时: {:.2}秒",
                    login_result.success, login_result.campus_status, login_result.elapsed_time
                );
                debug_info.write().response_content = response_content;
                message.set("Debug登录完成".to_string());
                message_center.log_event("INFO", "Debug登录完成").ok();
            }
            Err(e) => {
                debug_info.write().error_message = e.to_string();
                message.set("Debug登录失败".to_string());
                message_center
                    .log_event("ERROR", &format!("Debug登录失败: {}", e))
                    .ok();
            }
        }

        session_logs.set(debug_output);
    });
}
