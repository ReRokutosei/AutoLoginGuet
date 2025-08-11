use notify_rust::Notification;

/// 通知管理器
pub struct NotificationManager {}

impl NotificationManager {
    pub fn new() -> Self {
        NotificationManager {}
    }

    pub fn show(&self, title: &str, message: &str) -> Result<(), String> {
        Notification::new()
            .summary(title)
            .body(message)
            .show()
            .map_err(|e| e.to_string())
    }
}