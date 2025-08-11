use chrono::Local;
use std::fs;
use std::io::Write;
use std::path::Path;
use crate::core::config::LoggingConfig;

/// 日志管理器
pub struct LogManager {
    config: LoggingConfig,
}

impl LogManager {
    pub fn new(config: LoggingConfig) -> Self {
        LogManager { config }
    }

    pub fn log_event(&self, level: &str, message: &str) -> Result<(), String> {
        if !self.config.enable_logging {
            return Ok(());
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_entry = format!("{} - {} - {}\r\n", timestamp, level, message);
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_file_path)
            .map_err(|e| e.to_string())
            .and_then(|mut file| file.write_all(log_entry.as_bytes()).map_err(|e| e.to_string()))
    }

    pub fn clean_old_logs(&self) -> Result<(), String> {
        if !self.config.enable_logging {
            return Ok(());
        }

        if Path::new(&self.config.log_file_path).exists() {
            let content = fs::read_to_string(&self.config.log_file_path).map_err(|e| e.to_string())?;
            let cutoff_date = Local::now()
                .checked_sub_signed(chrono::Duration::days(self.config.info_log_retention_days))
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            
            let mut new_lines = Vec::new();
            for line in content.lines() {
                if line.len() >= 19 {
                    if let Ok(log_date) = chrono::NaiveDateTime::parse_from_str(&line[..19], "%Y-%m-%d %H:%M:%S") {
                        if let Ok(cutoff_date) = chrono::NaiveDateTime::parse_from_str(&cutoff_date, "%Y-%m-%d %H:%M:%S") {
                            if line.contains("INFO") && log_date < cutoff_date {
                                continue;
                            }
                        }
                    }
                }
                new_lines.push(format!("{}\r\n", line));
            }
            
            fs::write(&self.config.log_file_path, new_lines.join("")).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    
    pub fn read_logs(&self) -> Result<String, String> {
        if !self.config.enable_logging {
            return Ok(String::new());
        }
        
        if !Path::new(&self.config.log_file_path).exists() {
            return Ok(String::new());
        }
        
        fs::read_to_string(&self.config.log_file_path).map_err(|e| e.to_string())
    }
}