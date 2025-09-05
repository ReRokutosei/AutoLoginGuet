//! 事件驱动通信模块
//!
//! 定义事件类型和事件处理机制

use crate::core::message::{CampusNetworkStatus, WanStatus};
use std::sync::{Arc, Mutex};

/// 应用程序事件
#[derive(Debug, Clone)]
pub enum AppEvent<'a> {
    /// 网络状态检查完成
    NetworkStatusChecked {
        campus_status: CampusNetworkStatus,
        wan_status: WanStatus,
        message: &'a str,
    },
    /// 登录尝试完成
    LoginAttempted {
        success: bool,
        message: &'a str,
        elapsed_time: f64,
    },
    /// 配置加载完成
    ConfigLoaded { success: bool, message: &'a str },
    /// 配置保存完成
    ConfigSaved { success: bool, message: &'a str },
    /// 开机自启设置完成
    AutoStartSet {
        enabled: bool,
        success: bool,
        message: &'a str,
    },
    /// 系统通知显示
    NotificationShown { title: &'a str, message: &'a str },
}

/// GUI事件处理器消息
///
/// 用于在事件处理器和GUI主线程之间传递事件处理请求
#[derive(Debug)]
pub enum GuiEventHandlerMessage {
    /// 网络状态检查完成
    NetworkStatusChecked { message: String },
    /// 登录尝试完成
    LoginAttempted {
        success: bool,
        message: String,
        elapsed_time: f64,
    },
    /// 配置保存完成
    ConfigSaved { success: bool, message: String },
    /// 开机自启设置完成
    AutoStartSet {
        enabled: bool,
        success: bool,
        message: String,
    },
    /// 日志记录
    LogRecorded { level: String, message: String },
}

/// 事件处理器 trait
pub trait EventHandler: Send + Sync {
    /// 处理事件
    fn handle_event(&self, event: AppEvent);
}

/// 全局事件总线，用于统一处理所有应用事件
#[derive(Clone)]
pub struct EventBus {
    handlers: Arc<Mutex<Vec<Box<dyn EventHandler>>>>,
}

impl EventBus {
    /// 创建新的事件总线实例
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 注册事件处理器
    pub fn register_handler(&self, handler: Box<dyn EventHandler>) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push(handler);
    }

    /// 分发事件给所有注册的处理器
    pub fn dispatch(&self, event: AppEvent) {
        let handlers = self.handlers.lock().unwrap();
        for handler in handlers.iter() {
            handler.handle_event(event.clone());
        }
    }

    /// 处理事件
    pub fn handle_event(&self, event: AppEvent) {
        let handlers = self.handlers.lock().unwrap();
        for handler in handlers.iter() {
            handler.handle_event(event.clone());
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// 通知网络状态检查事件的通用函数
pub fn notify_network_status_checked(
    event_bus: &EventBus,
    campus_status: CampusNetworkStatus,
    wan_status: WanStatus,
    message: &str,
) {
    event_bus.dispatch(AppEvent::NetworkStatusChecked {
        campus_status,
        wan_status,
        message,
    });
}

/// 通知登录尝试事件的通用函数
pub fn notify_login_attempted(
    event_bus: &EventBus,
    success: bool,
    message: &str,
    elapsed_time: f64,
) {
    event_bus.dispatch(AppEvent::LoginAttempted {
        success,
        message,
        elapsed_time,
    });
}

/// 通知配置加载事件的通用函数
pub fn notify_config_loaded(event_bus: &EventBus, success: bool, message: &str) {
    event_bus.dispatch(AppEvent::ConfigLoaded { success, message });
}

/// 通知配置保存事件的通用函数
pub fn notify_config_saved(event_bus: &EventBus, success: bool, message: &str) {
    event_bus.dispatch(AppEvent::ConfigSaved { success, message });
}

/// 通知开机自启设置事件的通用函数
pub fn notify_auto_start_set(event_bus: &EventBus, enabled: bool, success: bool, message: &str) {
    event_bus.dispatch(AppEvent::AutoStartSet {
        enabled,
        success,
        message,
    });
}

/// 通知通知显示事件的通用函数
pub fn notify_notification_shown(event_bus: &EventBus, title: &str, message: &str) {
    event_bus.dispatch(AppEvent::NotificationShown { title, message });
}
