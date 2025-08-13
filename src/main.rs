#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod core;
pub mod gui;

use std::env;
use std::process;
use crate::core::config::{load_config, is_config_complete};
use crate::core::logging::LogManager;
use crate::core::notification::NotificationManager;
use crate::core::network::{NetworkManager, NetworkStatus};
use crate::core::{decrypt_password, generate_machine_key};

#[cfg(windows)]
fn set_auto_start(enabled: bool, _config: &core::config::ConfigData) -> Result<(), Box<dyn std::error::Error>> {
    use winreg::RegKey;
    use winreg::enums::*;

    let exe_path = env::current_exe()?;
    let exe_name = exe_path
        .file_name()
        .ok_or("无法获取可执行文件名")?
        .to_string_lossy()
        .to_string();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_READ | KEY_WRITE,
    )?;
    
    if enabled {
        let cmd = format!("\"{}\" -silent", exe_path.to_string_lossy());
        run_key.set_value(&exe_name, &cmd)?;
    } else {
        match run_key.delete_value(&exe_name) {
            Ok(_) => {},
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(Box::new(e));
                }
            }
        }
    }
    
    Ok(())
}

/// 检测是否已安装 WebView2
#[cfg(windows)]
fn is_webview2_installed() -> bool {
    use winreg::RegKey;
    use winreg::enums::*;

    const WEBVIEW2_CLIENT_ID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hklm_key_paths = [
        format!("SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{}", WEBVIEW2_CLIENT_ID),
        format!("SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{}", WEBVIEW2_CLIENT_ID),
    ];
    
    for hklm_key_path in &hklm_key_paths {
        if let Ok(key) = hklm.open_subkey(hklm_key_path) {
            // 如果能打开键，说明 WebView2 相关注册表项存在
            // 尝试获取 pv 值
            if let Ok(version) = key.get_value::<String, _>("pv") {
                // 检查版本号是否有效（不为空且不为 0）
                if !version.is_empty() && version != "0.0.0.0" {
                    return true;
                }
            } else {
                // 如果没有 pv 值，但键存在，也可能表示已安装
                // 某些情况下，键存在本身就表明已安装
                return true;
            }
        }
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hkcu_key_path = format!(
        "Software\\Microsoft\\EdgeUpdate\\Clients\\{}",
        WEBVIEW2_CLIENT_ID
    );
    
    if let Ok(key) = hkcu.open_subkey(&hkcu_key_path) {
        if let Ok(version) = key.get_value::<String, _>("pv") {
            // 检查版本号是否有效（不为空且不为 0）
            if !version.is_empty() && version != "0.0.0.0" {
                return true;
            }
        } else {
            // 如果没有 pv 值，但键存在，也可能表示已安装
            // 某些情况下，键存在本身就表明已安装
            return true;
        }
    }
    
    // 如果以上检查都没有通过，则认为未安装 WebView2
    false
}

#[cfg(windows)]
fn show_webview2_installation_guide() {
    use win_msgbox::Okay;
    let _ = win_msgbox::show::<Okay>(
        "未检测到 WebView2 Runtime，程序需要此组件才能正常运行。\n\
         如果您已安装 Microsoft Edge 浏览器，则应该可以正常运行。\n\
         否则请访问以下链接下载并安装 WebView2 Runtime：\n\
         https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/\
    ");
}

fn main() {
    #[cfg(windows)]
    {
        if !is_webview2_installed() {
            show_webview2_installation_guide();
            std::process::exit(1);
        }
    }
    
    let args: Vec<String> = env::args().collect();
    
    // 只有在明确指定-silent参数时才执行静默登录
    let is_silent_mode = args.contains(&"-silent".to_string());
    
    if is_silent_mode {
        match silent_login() {
            Ok(should_exit) => {
                if should_exit {
                    process::exit(0);
                } else {
                    gui::launch_gui();
                }
            }
            Err(e) => {
                eprintln!("发生错误: {}", e);
                gui::launch_gui();
            }
        }
    } else {
        gui::launch_gui();
    }
}

fn silent_login() -> Result<bool, String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建Runtime失败: {}", e))?;
    rt.block_on(async {
        match load_config().await {
            Ok(config) => silent_login_with_config(config).await,
            Err(e) => Err(format!("加载配置失败: {}", e))
        }
    })
}

pub async fn silent_login_with_config(config: core::config::ConfigData) -> Result<bool, String> {
    let start_time = std::time::Instant::now();
    let log_manager = LogManager::new(config.logging.clone());
    let notification_manager = NotificationManager::new();
    let network_manager = NetworkManager::new(config.network.clone());
    
    if let Err(e) = log_manager.clean_old_logs() {
        return Err(format!("清理旧日志失败: {}", e));
    }

    if !is_config_complete(&config) {
        return Ok(false);
    }

    match network_manager.check_network_status().await {
        Ok(status) => {
            let elapsed = start_time.elapsed().as_secs_f64();
            
            match status {
                NetworkStatus::LoggedInAndConnected => {
                    let log_message = format!("Network status: Device logged in and connected to WAN. Elapsed time: {:.2} seconds.", elapsed);
                    let status_msg = format!("已登录！网络连接正常\n用时{:.2}秒", elapsed);
                    if let Err(e) = log_manager.log_event("INFO", &log_message) {
                        return Err(format!("日志记录失败: {}", e));
                    }
                    if let Err(e) = notification_manager.show("当前网络状态", &status_msg) {
                        return Err(format!("通知显示失败: {}", e));
                    }
                    Ok(true)
                }
                NetworkStatus::NotLoggedInButConnected => {
                    // 解密密码
                    let machine_key = generate_machine_key();
                    let password = match decrypt_password(&config.account.encrypted_password, &machine_key) {
                        Ok(pwd) => pwd,
                        Err(e) => {
                            return Err(format!("解密密码失败: {}", e));
                        }
                    };
                    
                    match network_manager.attempt_login_with_credentials(
                        &config.account.username,
                        &password,
                        &config.account.isp,
                    ).await {
                        Ok(login_text) => {
                            let elapsed = start_time.elapsed().as_secs_f64();

                            // 添加对drcom接口返回的JSON格式成功响应的判断
                            if login_text.contains("注销页") || login_text.contains("认证成功页") || login_text.contains("Dr.COMWebLoginID_3.htm") || login_text.contains("\"result\":1") {
                                // 登录成功
                                let log_message = format!("Login successful. Elapsed time: {:.2} seconds.", elapsed);
                                let status_msg = format!("登录成功！已接入广域网\n用时{:.2}秒", elapsed);
                                if let Err(e) = log_manager.log_event("INFO", &log_message) {
                                    return Err(format!("日志记录失败: {}", e));
                                }
                                if let Err(e) = notification_manager.show("当前网络状态", &status_msg) {
                                    return Err(format!("通知显示失败: {}", e));
                                }
                                Ok(true)
                            } else if login_text.contains("msga='ldap auth error'") || 
                                      login_text.contains("ldap auth error") || 
                                      login_text.contains("Msg=01") {
                                // 账号密码或运营商错误
                                let error_type = if login_text.contains("Msg=01") {
                                    "Account or password error"
                                } else {
                                    "Authentication error (ldap auth error)"
                                };
                                
                                let log_message = format!("Login failed: {}. Elapsed time: {:.2} seconds.", error_type, elapsed);
                                let status_msg = if error_type.contains("Account or password") {
                                    format!("登录失败！请检查账号或密码\n用时{:.2}秒", elapsed)
                                } else {
                                    format!("登录失败！请检查登录信息\n用时{:.2}秒", elapsed)
                                };
                                
                                if let Err(e) = log_manager.log_event("WARNING", &log_message) {
                                    return Err(format!("日志记录失败: {}", e));
                                }
                                if let Err(e) = notification_manager.show("当前网络状态", &status_msg) {
                                    return Err(format!("通知显示失败: {}", e));
                                }
                                Ok(false)
                            } else {
                                // 其他登录失败情况
                                let log_message = format!("Login failed: Unknown error. Response snippet: {}. Elapsed time: {:.2} seconds.", 
                                                        &login_text[..std::cmp::min(200, login_text.len())], elapsed);
                                let status_msg = format!("登录失败，未知错误！\n用时{:.2}秒", elapsed);
                                if let Err(e) = log_manager.log_event("WARNING", &log_message) {
                                    return Err(format!("日志记录失败: {}", e));
                                }
                                if let Err(e) = notification_manager.show("当前网络状态", &status_msg) {
                                    return Err(format!("通知显示失败: {}", e));
                                }
                                Ok(false)
                            }
                        }
                        Err(e) => {
                            let elapsed = start_time.elapsed().as_secs_f64();
                            let _log_message = format!("Login request failed: {}. Elapsed time: {:.2} seconds.", e, elapsed);
                            // 判断是否为网络连接问题
                            if e.to_string().contains("error sending request for url") {
                                let status_msg = format!("网络连接失败，请检查网线或代理\n用时{:.2}秒", elapsed);
                                let detailed_msg = format!("网络连接失败: {}. Elapsed time: {:.2} seconds.", e, elapsed);
                                
                                if let Err(log_err) = log_manager.log_event("ERROR", &detailed_msg) {
                                    return Err(format!("日志记录失败: {}", log_err));
                                }
                                
                                if let Err(notify_err) = notification_manager.show("当前网络状态", &status_msg) {
                                    return Err(format!("通知显示失败: {}", notify_err));
                                }
                            } else {
                                let status_msg = format!("登录请求失败，用时{:.2}秒", elapsed);
                                let detailed_msg = format!("登录请求失败: {}. Elapsed time: {:.2} seconds.", e, elapsed);
                                
                                if let Err(log_err) = log_manager.log_event("ERROR", &detailed_msg) {
                                    return Err(format!("日志记录失败: {}", log_err));
                                }
                                
                                if let Err(notify_err) = notification_manager.show("当前网络状态", &status_msg) {
                                    return Err(format!("通知显示失败: {}", notify_err));
                                }
                            }
                            
                            Ok(false)
                        }
                    }
                }
                NetworkStatus::ConnectedToWan => {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let log_message = format!(
                        "Network status: Connected to WAN but GUET campus network status unknown. Elapsed time: {:.2} seconds.",
                        elapsed
                    );
                    let status_msg = format!("已连接广域网，校园网状态未知\n用时{:.2}秒", elapsed);
                                        
                    if let Err(e) = log_manager.log_event("WARNING", &log_message) {
                        return Err(format!("日志记录失败: {}", e));
                    }
                                        
                    if let Err(e) = notification_manager.show("当前网络状态", &status_msg) {
                        return Err(format!("通知显示失败: {}", e));
                    }
                                        
                    Ok(false)
                }
                NetworkStatus::NetworkCheckFailed | NetworkStatus::NoNetwork => {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let status_msg = format!("未连接到校园网，请检查网络连接\n用时{:.2}秒", elapsed);
                    
                    if let Err(e) = log_manager.log_event("WARNING", &format!(
                        "Network status: Not connected to campus network. Elapsed time: {:.2} seconds.",
                        elapsed
                    )) {
                        return Err(format!("日志记录失败: {}", e));
                    }
                    
                    if let Err(e) = notification_manager.show("当前网络状态", &status_msg) {
                        return Err(format!("通知显示失败: {}", e));
                    }
                    
                    Ok(false)
                }
            }
        }
        Err(e) => {
            let elapsed = start_time.elapsed().as_secs_f64();
            let status = format!("网络检查失败，用时{:.2}秒", elapsed);
            if let Err(e) = log_manager.log_event("ERROR", &format!(
                "Network status check failed: {}. Elapsed time: {:.2} seconds.",
                e, elapsed
            )) {
                return Err(format!("日志记录失败: {}", e));
            }
            if let Err(e) = notification_manager.show("当前网络状态", &status) {
                return Err(format!("通知显示失败: {}", e));
            }
            Ok(false)
        }
    }
}