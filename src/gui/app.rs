//! GUI应用程序主逻辑

use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use crate::core::config::{ConfigData, save_config, AccountConfig, SettingsConfig};
use crate::core::network::{NetworkManager, NetworkStatus};
use crate::core::logging::LogManager;
use crate::gui::debug::perform_debug_network_request;
use crate::core::{encrypt_password, generate_machine_key};
use base64;
use base64::Engine;

const GUET_LOGO: &[u8] = include_bytes!("../../assets/guet.jpg");

pub fn launch_gui() {
    use dioxus::desktop::tao::dpi::LogicalSize;
    
    let window = WindowBuilder::new()
        .with_title("AutoLoginGUET")
        .with_inner_size(LogicalSize::new(354.0, 550.0))
        .with_decorations(true);

    let config = Config::new()
        .with_window(window);

    LaunchBuilder::new()
        .with_cfg(config)
        .launch(app);
}

/// GUI配置结构体
#[derive(Clone, PartialEq, Default)]
pub struct GuiConfig {
    pub username: String,
    pub password: String,
    pub isp: String,
    pub auto_start: bool,
}

impl From<GuiConfig> for ConfigData {
    fn from(gui_config: GuiConfig) -> Self {
        gui_config_to_config_data(&gui_config)
    }
}

impl From<&GuiConfig> for ConfigData {
    fn from(gui_config: &GuiConfig) -> Self {
        gui_config_to_config_data(gui_config)
    }
}

impl From<ConfigData> for GuiConfig {
    fn from(config: ConfigData) -> Self {
        GuiConfig {
            username: config.account.username,
            password: String::new(), // 不直接显示加密密码
            isp: if config.account.isp.is_empty() {
                "校园网".to_string()
            } else {
                config.account.isp
            },
            auto_start: config.settings.auto_start,
        }
    }
}

pub fn gui_config_to_config_data(gui_config: &GuiConfig) -> ConfigData {
    let default = ConfigData::default();
    
    let machine_key = generate_machine_key();
    let encrypted_password = if !gui_config.password.is_empty() {
        encrypt_password(&gui_config.password, &machine_key).unwrap_or_else(|e| {
            eprintln!("密码加密失败: {}", e);
            String::new()
        })
    } else {
        String::new()
    };
    
    ConfigData {
        network: default.network,
        account: AccountConfig {
            username: gui_config.username.clone(),
            encrypted_password,
            isp: if gui_config.isp == "校园网" { 
                String::new() 
            } else { 
                gui_config.isp.clone() 
            },
        },
        settings: SettingsConfig {
            auto_start: gui_config.auto_start,
        },
        ..default
    }
}

fn app() -> Element {
    let mut gui_config = use_signal(|| GuiConfig::default());
    let mut message = use_signal(String::new);
    let mut logs = use_signal(String::new);
    let session_logs = use_signal(String::new);
    let debug_info = use_signal(|| crate::gui::debug::DebugInfo::default());
    let mut debug_mode = use_signal(|| false);
    
    use_effect(move || {
        spawn(async move {
            let config_result = crate::core::config::load_config().await;
            if let Ok(config) = config_result {
                let gui_config_data = GuiConfig::from(config);
                gui_config.set(gui_config_data);
            }
        });
        
        (|| ())()
    });
    
    use_effect(move || {
        spawn(async move {
            let config_result = crate::core::config::load_config().await;
            let config = match config_result {
                Ok(config) => config,
                Err(e) => {
                    message.set(format!("加载配置失败: {}", e));
                    return;
                }
            };
            
            let log_manager = LogManager::new(config.logging.clone());
            match log_manager.read_logs() {
                Ok(log_content) => {
                    logs.set(log_content);
                }
                Err(e) => {
                    message.set(format!("读取日志失败: {}", e));
                }
            }
            
            let network_manager = NetworkManager::new(config.network.clone());
            match network_manager.check_network_status().await {
                Ok(status) => {
                    let message_str = match status {
                        NetworkStatus::LoggedInAndConnected => {
                            "已登录校园网，并且网络正常".to_string()
                        }
                        NetworkStatus::NotLoggedInButConnected => {
                            "未登录校园网，但已连接其他网络".to_string()
                        }
                        NetworkStatus::ConnectedToWan => {
                            "已连接广域网".to_string()
                        }
                        NetworkStatus::NetworkCheckFailed => {
                            "网络检查失败".to_string()
                        }
                        NetworkStatus::NoNetwork => {
                            "无网络连接".to_string()
                        }
                    };
                    message.set(message_str);
                }
                Err(e) => {
                    message.set(format!("网络检查失败: {}", e));
                }
            }
        });
        
        (|| ())()
    });

    let on_username_input = move |e: Event<FormData>| {
        gui_config.write().username = e.value().clone();
    };
    
    let _on_password_input = move |e: Event<FormData>| {
        gui_config.write().password = e.value().clone();
    };
    
    
    let on_isp_change = move |e: Event<FormData>| {
        let value = e.value().clone();
        gui_config.write().isp = if value.is_empty() { 
            "校园网".to_string() 
        } else { 
            value 
        };
    };

    let on_auto_start_change = move |e: Event<FormData>| {
        gui_config.write().auto_start = e.value().parse().unwrap_or(false);
    };
    
    let on_save_config = move |_| {
        let gui_config = gui_config.clone();
        let message = message.clone();
        spawn(async move {
            let config_to_save = gui_config_to_config_data(&*gui_config.read());
            match save_config(&config_to_save) {
                Ok(_) => {
                    match crate::set_auto_start(gui_config().auto_start, &config_to_save) {
                        Ok(_) => {
                            let mut message = message.clone();
                            message.set("配置已保存".to_string());
                        }
                        Err(e) => {
                            let mut message = message.clone();
                            message.set(format!("配置已保存，但开机自启设置失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    let mut message = message.clone();
                    message.set(format!("保存配置失败: {}", e));
                }
            }
        });
        ()
    };
    
    let on_immediate_login = move |_| {
        if debug_mode() {
            let mut message = message.clone();
            let mut session_logs = session_logs.clone();
            message.set("正在Debug登录...".to_string());
            session_logs.write().clear();
            
            let config_to_login: ConfigData = (&*gui_config.read()).into();
            
            let mut debug_output = String::new();
            let debug_msg = format!("[{}] 开始Debug登录...\n", 
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
            debug_output.push_str(&debug_msg);
            
            perform_debug_network_request(
                config_to_login,
                debug_info,
                message,
                session_logs,
                debug_output,
            );
        } else {
            let mut message = message.clone();
            let mut session_logs = session_logs.clone();
            message.set("正在尝试登录...".to_string());
            session_logs.write().clear();
            
            spawn(async move {
                let mut message = message.clone();
                let mut session_logs = session_logs.clone();
                let gui_config = gui_config.clone();
                let mut logs = logs.clone();
                
                let config_to_login: ConfigData = (&*gui_config.read()).into();
                let start_time = std::time::Instant::now();
                
                let login_start_msg = format!("[{}] 开始登录尝试...\n", 
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
                session_logs.write().push_str(&login_start_msg);
                
                let login_result = crate::silent_login_with_config(config_to_login).await;
                match login_result {
                    Ok(success) => {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        if success {
                            message.set("登录成功!".to_string());
                            let success_msg = format!("[{}] 登录成功! 用时 {:.2} 秒\n", 
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), elapsed);
                            session_logs.write().push_str(&success_msg);
                        } else {
                            message.set("登录失败，请检查配置".to_string());
                            let fail_msg = format!("[{}] 登录失败! 用时 {:.2} 秒\n", 
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), elapsed);
                            session_logs.write().push_str(&fail_msg);
                        }
                        
                        let config_result = crate::core::config::load_config().await;
                        let config = match config_result {
                            Ok(config) => config,
                            Err(e) => {
                                message.set(format!("加载配置失败: {}", e));
                                let error_msg = format!("[{}] 加载配置失败: {}\n", 
                                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), e);
                                session_logs.write().push_str(&error_msg);
                                return;
                            }
                        };
                        
                        let log_manager = LogManager::new(config.logging.clone());
                        match log_manager.read_logs() {
                            Ok(log_content) => {
                                logs.set(log_content);
                            }
                            Err(e) => {
                                message.set(format!("读取日志失败: {}", e));
                                let error_msg = format!("[{}] 读取日志失败: {}\n", 
                                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), e);
                                session_logs.write().push_str(&error_msg);
                            }
                        }
                    }
                    Err(e) => {
                        message.set(format!("登录失败: {}", e));
                        let error_msg = format!("[{}] 登录失败: {}\n", 
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), e);
                        session_logs.write().push_str(&error_msg);
                    }
                }
            });
        }
    };
    
    let on_debug_mode_toggle = move |e: Event<FormData>| {
        let value = e.value().clone();
        spawn(async move {
            debug_mode.set(value.parse().unwrap_or(false));
        });
        ()
    };
    
    rsx! {
        style { {crate::gui::components::CSS} }
        div { class: "container",
            div { class: "app-title", "AUTO LOGIN GUET" }
            
            div { class: "avatar-container",
                img { 
                    class: "avatar", 
                    src: "data:image/jpeg;base64,{base64::engine::general_purpose::STANDARD.encode(GUET_LOGO)}",
                    alt: "用户头像"
                }
            }
            
            div { class: "form-group",
                div { class: "form-row",
                    input {
                        r#type: "text",
                        value: "{gui_config().username}",
                        oninput: on_username_input,
                        placeholder: "学号"
                    }
                }
                
                div { class: "form-row",
                    input {
                        r#type: "password",
                        value: "{gui_config.read().password}",
                        oninput: move |e: Event<FormData>| {
                            gui_config.write().password = e.value().clone();
                        },
                        placeholder: "密码"
                    }
                }
            }
            
            div { class: "form-group",
                div { class: "select-row",
                    select {
                        value: "{gui_config().isp}",
                        onchange: on_isp_change,
                        option { value: "", "校园网" }
                        option { value: "@cmcc", "中国移动" }
                        option { value: "@unicom", "中国联通" }
                        option { value: "@telecom", "中国电信" }
                    }
                }
            }
            
            div { class: "checkbox-group",
                div { class: "checkbox-item",
                    input {
                        r#type: "checkbox",
                        checked: gui_config().auto_start,
                        oninput: on_auto_start_change,
                        title: "勾选并点击[保存配置]后生效\n启用后，程序将在电脑开机时静默登录校园网（推荐勾选）\n启用前，必须先填写并保存配置信息"
                    }
                    label { "开机自启" }
                }
                div { class: "checkbox-item",
                    input {
                        r#type: "checkbox",
                        checked: debug_mode(),
                        oninput: on_debug_mode_toggle,
                        title: "启用后，程序将输出原始调试信息（仅供开发者使用）"
                    }
                    label { "Debug模式" }
                }
            }
            
            div { class: "button-group",
                button {
                    class: "btn btn-primary",
                    onclick: on_save_config,
                    "保存配置"
                }
                
                button {
                    class: "btn btn-success",
                    onclick: on_immediate_login,
                    "立即登录"
                }
            }
            
            if !message().is_empty() {
                div { class: "alert alert-info", "{message}" }
            }
            
            div { class: "form-group",
                label { "当前会话日志:" }
                textarea {
                    class: "session-logs",
                    readonly: true,
                    value: "{session_logs}",
                    placeholder: "登录操作的日志将显示在这里"
                }
            }
        }
    }
}