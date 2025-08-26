//! GUI应用程序模块

use dioxus::prelude::*;
use std::sync::mpsc::Receiver;
use base64::Engine;
use dioxus::desktop::tao::dpi::PhysicalPosition;
use dioxus::desktop::use_window;

use autologinguet_core::core::service::{AuthService, validate_username, validate_password};
use autologinguet_core::core::dto::GuiConfigDto;
use autologinguet_core::core::events::GuiEventHandlerMessage;
use crate::gui::debug::{DebugInfo, perform_debug_network_request};
use crate::gui::init::{init_app_config, init_app_logs_and_network};
use crate::gui::state::GuiConfigWithData;
use crate::gui::gui_event::process_gui_events;

/// GUI主应用组件
fn app() -> Element {
    let mut gui_config = use_signal(GuiConfigDto::default);
    let gui_config_with_data = use_signal(GuiConfigWithData::default);
    let mut message = use_signal(String::new);
    let logs = use_signal(String::new);
    let mut session_logs = use_signal(String::new);
    let debug_info = use_signal(DebugInfo::default);
    let auth_service = use_signal(|| Option::<AuthService>::None);
    let receiver: Signal<Option<Receiver<GuiEventHandlerMessage>>> = use_signal(|| None);
    // 添加用于跟踪输入验证状态的信号
    let mut username_invalid = use_signal(|| false);
    let mut password_invalid = use_signal(|| false);
    
    // 居中窗口
    use_effect(move || {
        let window = use_window();

        spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            
            if let Some(monitor) = window.current_monitor() {
                let monitor_size = monitor.size();
                let window_size = window.outer_size();

                let new_position = PhysicalPosition::new(
                    (monitor_size.width as i32 - window_size.width as i32) / 2,
                    (monitor_size.height as i32 - window_size.height as i32) / 2,
                );

                window.set_outer_position(new_position);
            }
        });
    });
    
    use_future(move || async move {
        let mut message = message;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if let Some(ref rcv) = *receiver.read() {
                process_gui_events(rcv, &mut message);
            }
        }
    });
    
    init_app_config(
        gui_config,
        gui_config_with_data,
        auth_service,
        receiver,
    );
    
    init_app_logs_and_network(
        auth_service,
        message,
        logs,
    );

    let on_username_input = move |e: Event<FormData>| {
        gui_config.write().username = e.value();
        username_invalid.set(false);
    };

    let on_username_blur = move |_| {
        let current_value = gui_config.read().username.clone();
        if !validate_username(&current_value) && !current_value.is_empty() {
            *message.write() = "账号格式不正确，请输入3-12位数字".to_string();
            username_invalid.set(true);
        } else {
            *message.write() = String::new();
        }
    };

    let on_password_input = move |e: Event<FormData>| {
        gui_config.write().password = e.value();
        password_invalid.set(false);
    };

    let on_password_blur = move |_| {
        let current_value = gui_config.read().password.clone();
        if !validate_password(&current_value) && !current_value.is_empty() {
            *message.write() = "密码必须包含大小写字母、数字和符号，长度8-32位".to_string();
            password_invalid.set(true);
        } else {
            *message.write() = String::new();
        }
    };
    
    let on_isp_select = move |e: Event<FormData>| {
        gui_config.write().isp = e.value();
    };
    
    let on_immediate_login = move |_| {
        if debug_info().enable_debug {
            *message.write() = "正在Debug登录...".to_string();
            session_logs.write().clear();
            
            let mut current_gui_config = gui_config();
            if current_gui_config.password.is_empty() && !gui_config_with_data().encrypted_password.is_empty() {
                current_gui_config.encrypted_password = gui_config_with_data().encrypted_password.clone();
            }
            
            let mut debug_output = String::new();
            let debug_msg = format!("[{}] 开始Debug登录...\n", 
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
            debug_output.push_str(&debug_msg);
            
            perform_debug_network_request(
                current_gui_config,
                debug_info,
                message,
                session_logs,
                debug_output,
            );

        } else {
            spawn(async move {
                if let Some(ref service) = *auth_service.read() {
                    let _ = crate::gui::gui_service::perform_login(service, &gui_config(), &gui_config_with_data(), message).await;
                } else {
                    *message.write() = "认证服务未初始化".to_string();
                }
            });
        }
    };

    let on_auto_start_toggle = move |e: Event<FormData>| {
        let new_value = e.value() == "true";
        
        #[cfg(windows)]
        spawn(async move {
            if let Some(ref service) = *auth_service.read() {
                match crate::gui::gui_service::set_auto_start(service, new_value, &gui_config(), &gui_config_with_data()).await {
                    Ok(_) => {
                        gui_config.write().auto_start = new_value;
                        *message.write() = "开机自启设置已更新".to_string();
                    }
                    Err(e) => {
                        *message.write() = format!("设置开机自启失败: {}", e);
                    }
                }
            } else {
                *message.write() = "认证服务未初始化".to_string();
            }
        });
        
    };

    rsx! {
        div { 
            style { {include_str!("../../assets/style.css")} }
            div { class: "container",
                div { class: "top-right-icons",
                    a { 
                        class: "icon-link",
                        href: "https://github.com/ReRokutosei/AutoLoginGuet",
                        target: "_blank",
                        title: "项目地址",
                        img { 
                            src: "data:image/svg+xml;base64,{base64::engine::general_purpose::STANDARD.encode(include_bytes!(\"../../assets/github.svg\"))}",
                            class: "icon-svg",
                        }
                    }
                    a { 
                        class: "icon-link",
                        href: "https://nicdrcom.guet.edu.cn/Self/unlogin/forgetPwd",
                        target: "_blank",
                        title: "重置密码",
                        img { 
                            src: "data:image/svg+xml;base64,{base64::engine::general_purpose::STANDARD.encode(include_bytes!(\"../../assets/key.svg\"))}",
                            class: "icon-svg",
                        }
                    }
                }
                
                div { class: "avatar-container",
                    img { 
                        class: "avatar", 
                        src: "data:image/jpeg;base64,{base64::engine::general_purpose::STANDARD.encode(include_bytes!(\"../../assets/guet.jpg\"))}",
                        alt: "用户头像"
                    }
                }
                
                div { class: "form-group",
                    div { class: "form-row",
                        input {
                            r#type: "text",
                            value: "{gui_config().username}",
                            class: if *username_invalid.read() { "invalid" } else { "" },
                            oninput: on_username_input,
                            onblur: on_username_blur,
                            placeholder: "输入学号或工号"
                        }
                    }
                }
                
                div { class: "form-group",
                    div { class: "form-row",
                        div { class: "password-container",
                            input {
                                r#type: "password",
                                value: "{gui_config.read().password}",
                                class: if *password_invalid.read() { "invalid" } else { "" },
                                oninput: on_password_input,
                                onblur: on_password_blur,
                                
                            }
                            div {
                                class: "password-hint",
                                match (gui_config.read().password.is_empty(), !gui_config_with_data.read().encrypted_password.is_empty()) {
                                    (true, true) => "当前已记录密码\n出于安全考虑不予显示",
                                    (true, false) => "输入密码",
                                    _ => ""
                                }
                            }
                        }
                    }
                }
                div { class: "form-group",
                    div { class: "select-row",
                        select {
                            value: "{gui_config().isp}",
                            onchange: on_isp_select,
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
                            checked: "{gui_config().auto_start}",
                            onchange: on_auto_start_toggle,
                            title: "（推荐勾选）\n启用前，必须先填写并保存配置信息\n启用后，程序将在电脑开机时静默登录校园网"
                        }
                        label { "开机自启" }
                    }
                    div { class: "checkbox-item",
                        input {
                            r#type: "checkbox",
                            checked: "{debug_info().enable_debug}",
                            onchange: move |e: Event<FormData>| {
                                let value = e.value() == "true";
                                let mut debug_info = debug_info;
                                spawn(async move {
                                    debug_info.write().enable_debug = value;
                                    if !value {
                                        session_logs.write().clear();
                                    }
                                });
                            },
                            title: "启用后，程序将输出原始调试信息（仅供开发者使用）"
                        }
                        label { "调试模式" }
                    }
                }
                
                // 仅在调试模式下显示日志框
                if debug_info().enable_debug {
                    div { class: "form-group",
                        label { "当前会话日志:" }
                        textarea {
                            class: "session-logs",
                            readonly: true,
                            value: "{session_logs}",
                            placeholder: "调试日志将显示在这里"
                        }
                    }
                }

                div { class: "button-group",
                    button { 
                        class: "btn btn-success", 
                        onclick: on_immediate_login,
                        "立即登录"
                    }
                }
                
                if !message().is_empty() {
                    div { class: "alert alert-info", "{message}" }
                }
            }
        }
    }
}

/// 启动GUI
pub fn launch_gui() {
    use dioxus::prelude::LaunchBuilder;
    use dioxus::desktop::{Config, WindowBuilder};
    use dioxus::desktop::tao::dpi::LogicalSize;
    
    let window = WindowBuilder::new()
        .with_title("AutoLoginGUET")
        .with_position(PhysicalPosition::new(100000, 100000))
        // 拼尽全力、用尽各种办法，都消除不了GUI启动时的一闪而过...
        .with_inner_size(LogicalSize::new(320.0, 465.0));

    let config = Config::new()
        .with_window(window)
        .with_menu(None);

    LaunchBuilder::new()
        .with_cfg(config)
        .launch(app);
}
