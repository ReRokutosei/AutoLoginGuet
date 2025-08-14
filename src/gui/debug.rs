//! 调试模块

use dioxus::prelude::*;
use crate::core::config::ConfigData;
use crate::core::network::NetworkManager;
use crate::core::{decrypt_password, generate_machine_key};

const DEFAULT_LOGIN_IP: &str = "http://10.0.1.5/";

#[derive(Clone, PartialEq, Default)]
pub struct DebugInfo {
    pub request_url: String,
    pub request_params: String,
    pub response_content: String,
    pub status_code: String,
    pub error_message: String,
}

pub fn perform_debug_login(
    config_to_login: ConfigData,
    mut debug_info: Signal<DebugInfo>,
    mut message: Signal<String>,
    _session_logs: Signal<String>,
) -> String {
    debug_info.write().error_message.clear();
    debug_info.write().response_content.clear();
    debug_info.write().request_params.clear();
    debug_info.write().request_url.clear();
    debug_info.write().status_code.clear();
    
    let mut debug_output = String::new();
    
    let debug_msg = format!("[{}] 开始Debug登录...\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    debug_output.push_str(&debug_msg);
    
    let machine_key = generate_machine_key();
    let password = match decrypt_password(&config_to_login.account.encrypted_password, &machine_key) {
        Ok(pwd) => pwd,
        Err(e) => {
            eprintln!("解密密码失败: {}", e);
            String::new()
        }
    };
    
    let actual_isp = if config_to_login.account.isp == "校园网" {
        ""
    } else {
        &config_to_login.account.isp
    };

    debug_info.write().request_url = DEFAULT_LOGIN_IP.to_string();
    debug_info.write().request_params = format!("callback=dr1003&DDDDD={}{}&upass=******&0MKKey=123456", config_to_login.account.username, actual_isp);

    debug_output.push_str(&format!("请求URL: {}\n", DEFAULT_LOGIN_IP));
    debug_output.push_str(&format!("实际发送的请求参数: callback=dr1003&DDDDD={}{}&upass={}&0MKKey=123456\n", config_to_login.account.username, actual_isp, password));
    debug_output.push_str("发送登录请求中...\n");
    
    message.set("正在进行Debug登录...".to_string());
    
    debug_output
}

pub fn perform_debug_network_request(
    config_to_login: ConfigData,
    mut debug_info: Signal<DebugInfo>,
    mut message: Signal<String>,
    mut session_logs: Signal<String>,
    current_log: String,
) {
    spawn(async move {
        let mut debug_output = current_log;
        let network_manager = NetworkManager::new(config_to_login.network.clone());
        
        let machine_key = generate_machine_key();
        let password = match decrypt_password(&config_to_login.account.encrypted_password, &machine_key) {
            Ok(pwd) => pwd,
            Err(e) => {
                eprintln!("解密密码失败: {}", e);
                String::new()
            }
        };
        
        let actual_isp = if config_to_login.account.isp == "校园网" {
            ""
        } else {
            &config_to_login.account.isp
        };
        
        debug_info.write().request_url = DEFAULT_LOGIN_IP.to_string();
        debug_info.write().request_params = format!("callback=dr1003&DDDDD={}{}&upass=******&0MKKey=123456", config_to_login.account.username, actual_isp);
        
        debug_output.push_str(&format!("使用客户端配置: {:?}\n", config_to_login.network));
        
        debug_output.push_str("构建HTTP客户端...\n");
        debug_output.push_str("添加请求头:\n");
        debug_output.push_str("  User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36\n");
        debug_output.push_str("  Referer: http://10.0.1.5/\n");
        debug_output.push_str("准备发送GET请求...\n");
        
        let login_result = network_manager.attempt_login_with_credentials(
                &config_to_login.account.username,
                &password,
                &config_to_login.account.isp,
            )
            .await;
        
        match login_result {
            Ok(response_content) => {
                debug_info.write().response_content = response_content.clone();
                message.set("Debug登录完成".to_string());
                debug_output.push_str(&format!("[{}] 收到响应\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
                debug_output.push_str("响应预览:\n");
                debug_output.push_str(&response_content);
                debug_output.push_str(&format!("\n[{}] Debug登录完成\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
            }
            Err(e) => {
                debug_info.write().error_message = e.clone();
                message.set("Debug登录失败".to_string());
                debug_output.push_str(&format!("[{}] 请求失败\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
                debug_output.push_str(&format!("错误信息: {}\n", e));
                debug_output.push_str(&format!("\n[{}] Debug登录失败\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
            }
        }
        
        session_logs.set(debug_output);
    });
}