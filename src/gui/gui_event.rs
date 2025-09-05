//! GUI事件处理器

use autologinguet_core::MessageCenter;
use autologinguet_core::core::events::{AppEvent, EventHandler, GuiEventHandlerMessage};
use dioxus::prelude::*;
use std::sync::mpsc::{self, Receiver, Sender};

/// 处理GUI事件消息，在GUI主线程中调用
pub fn process_gui_events(
    receiver: &Receiver<GuiEventHandlerMessage>,
    message: &mut Signal<String>,
) {
    for msg in receiver.try_iter() {
        match msg {
            GuiEventHandlerMessage::NetworkStatusChecked { message: msg } => {
                message.write().clone_from(&msg);
            }
            GuiEventHandlerMessage::LoginAttempted {
                success: _,
                message: msg,
                elapsed_time: _,
            } => {
                message.write().clone_from(&msg);
            }
            GuiEventHandlerMessage::ConfigSaved {
                success: _,
                message: msg,
            } => {
                message.write().clone_from(&msg);
            }
            GuiEventHandlerMessage::AutoStartSet {
                enabled: _,
                success: _,
                message: msg,
            } => {
                message.write().clone_from(&msg);
            }
            GuiEventHandlerMessage::LogRecorded {
                level,
                message: msg,
            } => {
                let message_center = MessageCenter::default();
                let _ = message_center.log_event(&level, &msg);
            }
        }
    }
}

/// GUI事件处理器
///
/// 用于处理来自核心模块的事件，并通过channel发送到GUI主线程更新状态
pub struct GuiEventHandler {
    /// 消息发送器
    sender: Sender<GuiEventHandlerMessage>,
}

impl GuiEventHandler {
    /// 创建新的GUI事件处理器
    pub fn new() -> (Self, Receiver<GuiEventHandlerMessage>) {
        let (sender, receiver) = mpsc::channel();

        let handler = Self { sender };

        (handler, receiver)
    }
}

impl EventHandler for GuiEventHandler {
    fn handle_event(&self, event: AppEvent) {
        match event {
            AppEvent::NetworkStatusChecked {
                campus_status: _,
                wan_status: _,
                message,
            } => {
                let _ = self
                    .sender
                    .send(GuiEventHandlerMessage::NetworkStatusChecked {
                        message: message.to_string(),
                    });
            }
            AppEvent::LoginAttempted {
                success,
                message,
                elapsed_time,
            } => {
                let _ = self.sender.send(GuiEventHandlerMessage::LoginAttempted {
                    success,
                    message: message.to_string(),
                    elapsed_time,
                });
            }
            AppEvent::ConfigSaved { success, message } => {
                let _ = self.sender.send(GuiEventHandlerMessage::ConfigSaved {
                    success,
                    message: message.to_string(),
                });
            }
            AppEvent::AutoStartSet {
                enabled,
                success,
                message,
            } => {
                let _ = self.sender.send(GuiEventHandlerMessage::AutoStartSet {
                    enabled,
                    success,
                    message: message.to_string(),
                });
            }
            AppEvent::NotificationShown { title: _, message } => {
                let _ = self.sender.send(GuiEventHandlerMessage::LogRecorded {
                    level: "INFO".to_string(),
                    message: message.to_string(),
                });
            }
            _ => {}
        }
    }
}
