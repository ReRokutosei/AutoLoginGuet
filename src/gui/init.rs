//! GUI初始化模块

use dioxus::prelude::*;
use autologinguet_core::{GuiConfigDto, AuthService};
use autologinguet_core::core::events::GuiEventHandlerMessage;
use std::sync::mpsc::Receiver;
use crate::gui::state::GuiConfigWithData;
use crate::gui::gui_service::{init_config, init_logs_and_network};

/// 初始化配置
///
/// 该函数用于初始化应用程序的配置信息，包括认证服务和事件接收器。
///
/// 参数：
/// - `gui_config`: GUI配置信息的信号
/// - `gui_config_with_data`: 带有额外数据的GUI配置信息的信号
/// - `mut auth_service`: 认证服务的信号（输出）
/// - `mut receiver`: 事件接收器的信号（输出）
///
/// 逻辑：
/// 1. 使用 `use_effect` 监听配置变化。
/// 2. 异步调用 `init_config` 获取认证服务和事件接收器。
/// 3. 如果成功获取，更新 `auth_service` 和 `receiver`。
pub fn init_app_config(
    gui_config: Signal<GuiConfigDto>,
    gui_config_with_data: Signal<GuiConfigWithData>,
    mut auth_service: Signal<Option<AuthService>>,
    mut receiver: Signal<Option<Receiver<GuiEventHandlerMessage>>>,
) {
    use_effect(move || {
        spawn(async move {
            if let Some((service, rcv)) = init_config(gui_config, gui_config_with_data).await {
                *auth_service.write() = Some(service);
                *receiver.write() = Some(rcv);
            }
        });
    });
}

/// 初始化日志和网络状态
///
/// 该函数用于初始化应用程序的日志和网络状态。
///
/// 参数：
/// - `auth_service`: 认证服务的信号
/// - `message`: 消息的信号（用于更新日志消息）
/// - `logs`: 日志的信号（用于更新日志内容）
///
/// 逻辑：
/// 1. 使用 `use_effect` 监听认证服务的变化。
/// 2. 如果认证服务存在，则异步调用 `init_logs_and_network` 初始化日志和网络状态。
pub fn init_app_logs_and_network(
    auth_service: Signal<Option<AuthService>>,
    message: Signal<String>,
    logs: Signal<String>,
) {
    use_effect(move || {
        spawn(async move {
            if let Some(ref service) = *auth_service.read() {
                init_logs_and_network(service, message, logs).await;
            }
        });
    });
}