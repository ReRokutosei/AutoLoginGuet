//! 配置管理器

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use crate::core::config::{ConfigData, save_config as save_config_to_file};

pub struct ConfigManager {
    last_save_time: Arc<Mutex<Option<Instant>>>,
    debounce_delay: Duration,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            last_save_time: Arc::new(Mutex::new(None)),
            debounce_delay: Duration::from_millis(500), // 500ms 防抖延迟
        }
    }

    pub async fn save_config_with_debounce(&self, config: ConfigData) -> Result<(), String> {
        let mut last_save_time = self.last_save_time.lock().await;
        
        let now = Instant::now();
        if let Some(last_time) = *last_save_time {
            if now.duration_since(last_time) < self.debounce_delay {
                return Ok(());
            }
        }
        
        *last_save_time = Some(now);

        save_config_to_file(&config)
    }
    

    pub fn save_config_immediately(&self, config: &ConfigData) -> Result<(), String> {
        save_config_to_file(config)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}