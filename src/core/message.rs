//! 消息处理中心模块
//!
//! 集中管理所有消息的生成、日志记录和通知显示

use crate::core::config::LoggingConfig;
use crate::core::error::{AppError, AppResult};
use crate::core::events::{notify_login_attempted, notify_network_status_checked, EventBus};
use crate::core::network::NetworkStatus;
use crate::core::service::LoginResult;
use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use notify_rust::Notification;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};


/// 消息构建器
pub struct MessageBuilder;

impl MessageBuilder {
    /// 格式化带耗时信息的消息
    /// 
    /// # 参数
    /// * `message` - 基础消息
    /// * `elapsed` - 花费时间（秒），如果为None则不包含时间信息
    pub fn format_message(message: &str, elapsed: Option<f64>) -> String {
        if let Some(elapsed_val) = elapsed {
            format!("{} 用时{:.2}秒", message, elapsed_val)
        } else {
            message.to_string()
        }
    }
}

impl Default for MessageCenter {
    /// 创建默认的消息处理中心实例（无日志配置）
    fn default() -> Self {
        Self {
            config: None,
            write_mutex: Arc::new(Mutex::new(())),
            event_bus: EventBus::new(),
        }
    }
}

/// 消息处理中心
#[derive(Clone)]
pub struct MessageCenter {
    config: Option<LoggingConfig>,
    /// 用于同步日志写入操作的互斥锁
    write_mutex: Arc<Mutex<()>>,
    event_bus: EventBus,
}

impl MessageCenter {
    /// 创建新的消息处理中心实例
    pub fn new(
        logging_config: Option<LoggingConfig>,
        event_bus: EventBus,
    ) -> Self {
        Self {
            config: logging_config,
            write_mutex: Arc::new(Mutex::new(())),
            event_bus,
        }
    }

    /// 处理网络状态检查结果
    /// 
    /// 该方法会：
    /// 1. 生成网络状态消息
    /// 2. 记录日志（如果启用）
    /// 3. 显示通知（如果启用）
    /// 4. 触发相关事件
    /// 
    /// # 参数
    /// * `status` - 网络状态信息
    /// * `elapsed` - 检查所花费的时间（秒）
    /// * `show_notification` - 是否显示通知
    /// * `should_log` - 是否记录日志
    /// 
    /// # 返回值
    /// 返回生成的状态消息字符串（用于GUI显示，不包含时间信息）
    pub fn handle_network_status(&self, status: &NetworkStatus, elapsed: f64, show_notification: bool, should_log: bool) -> String {
        // 生成用于日志和通知的带时间信息的消息
        let full_message = MessageBuilder::format_message(&status.to_message(), Some(elapsed));
        
        // 生成用于GUI显示的不带时间信息的消息
        let gui_message = MessageBuilder::format_message(&status.to_message(), None);

        if should_log {
            let log_message = format!("{} 用时{:.2}秒", 
                                     status.to_message(), elapsed);
            let _ = self.log_event("INFO", &log_message);
        }

        if show_notification {
            let _ = self.show_notification("", &full_message);
        }

        notify_network_status_checked(&self.event_bus, status.clone(), &gui_message);

        gui_message
    }


    /// 处理登录结果并执行相关操作
    /// 
    /// 该方法会：
    /// 1. 触发登录尝试事件
    /// 2. 记录日志
    /// 3. 显示通知
    /// 4. 触发相关事件
    /// 
    /// # 参数
    /// * `result` - 登录结果
    /// 
    /// # 返回值
    /// 返回处理后的LoginResult对象，其中的消息用于GUI显示（不包含时间信息）
    pub fn handle_login_result(&self, result: LoginResult) -> LoginResult {
        notify_login_attempted(&self.event_bus, result.success, &result.message, result.elapsed_time);
        
        self.handle_login_result_internal(result)
    }

    /// 处理登录结果但不触发事件（用于内部调用）
    /// 
    /// # 参数
    /// * `result` - 登录结果
    /// 
    /// # 返回值
    /// 返回处理后的LoginResult对象，其中的消息用于GUI显示（不包含时间信息）
    pub fn handle_login_result_without_event(&self, result: LoginResult) -> LoginResult {
        self.handle_login_result_internal(result)
    }

    /// 处理登录结果的内部实现
    /// 
    /// 该方法会：
    /// 1. 记录日志
    /// 2. 显示通知
    /// 3. 触发相关事件（除了LoginAttempted）
    /// 
    /// # 参数
    /// * `result` - 登录结果
    /// 
    /// # 返回值
    /// 返回处理后的LoginResult对象，其中的消息用于GUI显示（不包含时间信息）
    pub fn handle_login_result_internal(&self, result: LoginResult) -> LoginResult {
        // 生成用于日志和通知的带时间信息的消息
        let full_message = MessageBuilder::format_message(&result.message, Some(result.elapsed_time));
        
        // 生成用于GUI显示的不带时间信息的消息
        let gui_message = MessageBuilder::format_message(&result.message, None);

        let log_level = if result.success { "INFO" } else { "ERROR" };
        let log_message = format!("{} 用时{:.2}秒",
                                 result.message, result.elapsed_time);
        let _ = self.log_event(log_level, &log_message);

        // 显示通知（包含时间信息）
        let _ = self.show_notification("", &full_message);

        // 更新LoginResult中的消息为不带时间的版本，用于GUI显示
        LoginResult {
            success: result.success,
            message: gui_message,
            elapsed_time: result.elapsed_time,
        }
    }
    
    /// 显示通知
    pub fn show_notification(&self, _title: &str, message: &str) -> AppResult<()> {
        // 统一将消息内容作为通知标题显示，忽略传入的标题参数
        Notification::new()
            .summary(message)
            .show()
            .map_err(|e| AppError::NotificationError(e.to_string()))
    }
    
    /// 生成带时间信息的消息文本
    /// 
    /// # 参数
    /// * `message` - 基础消息
    /// * `elapsed` - 花费时间（秒），如果为None则不包含时间信息
    pub fn format_message(&self, message: &str, elapsed: Option<f64>) -> String {
        MessageBuilder::format_message(message, elapsed)
    }
    
    /// 获取日志文件路径
    pub fn get_log_file_path(&self) -> Option<&str> {
        self.config.as_ref().map(|c| c.log_file_path.as_str())
    }

    /// 记录日志事件
    pub fn log_event(&self, level: &str, message: &str) -> AppResult<()> {
        if self.config.as_ref().is_none_or(|c| !c.enable_logging) {
            return Ok(());
        }

        // 获取互斥锁
        let _guard = self.write_mutex.lock().unwrap();

        let log_file_path = self.get_log_file_path().unwrap();
        let log_entry = format!("[{}][{}] {}\n", 
                               Local::now().format("%Y-%m-%d %H:%M:%S"),
                               level,
                               message);

        if let Some(parent) = Path::new(log_file_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::SystemError(format!("无法创建日志目录: {}", e)))?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .map_err(|e| AppError::SystemError(format!("无法打开日志文件 '{}': {}", log_file_path, e)))?;

        file.write_all(log_entry.as_bytes())
            .map_err(|e| AppError::SystemError(format!("无法写入日志文件: {}", e)))?;

        Ok(())
    }

    /// 读取日志
    pub fn read_logs(&self) -> AppResult<String> {
        // 只读操作不需要锁
        let log_file_path = match self.get_log_file_path() {
            Some(path) => path,
            None => return Ok(String::new()),
        };
        
        if !Path::new(log_file_path).exists() {
            return Ok(String::new());
        }

        let file = File::open(log_file_path)
            .map_err(|e| AppError::SystemError(format!("无法打开日志文件 '{}': {}", log_file_path, e)))?;
        
        let reader = BufReader::new(file);
        let mut logs = String::new();
        
        for line in reader.lines() {
            logs.push_str(&line.map_err(|e| AppError::SystemError(format!("读取日志行失败: {}", e)))?);
            logs.push('\n');
        }
        
        Ok(logs)
    }

    /// 清理旧日志
    pub fn clean_old_logs(&self) -> AppResult<()> {
        if self.config.as_ref().is_none_or(|c| !c.enable_logging) {
            return Ok(());
        }

        // 获取写入锁
        let _guard = self.write_mutex.lock().unwrap();

        let log_file_path = self.get_log_file_path().unwrap();
        
        if !Path::new(log_file_path).exists() {
            return Ok(());
        }

        let cutoff_date = Local::now() - Duration::days(self.config.as_ref().unwrap().info_log_retention_days);
        let temp_file_path = format!("{}.tmp", log_file_path);

        {
            let input_file = File::open(log_file_path)
                .map_err(|e| AppError::SystemError(format!("无法打开日志文件 '{}': {}", log_file_path, e)))?;
            let output_file = File::create(&temp_file_path)
                .map_err(|e| AppError::SystemError(format!("无法创建临时文件 '{}': {}", temp_file_path, e)))?;

            let reader = BufReader::new(input_file);
            let mut writer = io::BufWriter::new(output_file);

            for line in reader.lines() {
                let line = line.map_err(|e| AppError::SystemError(format!("读取日志行失败: {}", e)))?;

                if let Some(date_part) = line.get(1..20) {  // [YYYY-MM-DD HH:MM:SS]
                    if let Ok(log_date) = NaiveDateTime::parse_from_str(date_part, "%Y-%m-%d %H:%M:%S") {
                        let log_date = Local.from_local_datetime(&log_date).unwrap();
                        if log_date >= cutoff_date {
                            writeln!(writer, "{}", line)
                                .map_err(|e| AppError::SystemError(format!("写入临时文件失败: {}", e)))?;
                        }
                    } else {
                        writeln!(writer, "{}", line)
                            .map_err(|e| AppError::SystemError(format!("写入临时文件失败: {}", e)))?;
                    }
                } else {
                    writeln!(writer, "{}", line)
                        .map_err(|e| AppError::SystemError(format!("写入临时文件失败: {}", e)))?;
                }
            }

            writer.flush()
                .map_err(|e| AppError::SystemError(format!("刷新临时文件失败: {}", e)))?;
        }

        // 替换原文件并清理临时文件
        fs::rename(&temp_file_path, log_file_path)
            .map_err(|e| AppError::SystemError(format!("替换日志文件失败: {}", e)))?;
        
        // 删除临时文件
        let _ = fs::remove_file(&temp_file_path);

        Ok(())
    }

    /// 通用日志记录函数
    /// 
    /// # 参数
    /// * `level` - 日志级别
    /// * `message` - 日志消息
    /// * `elapsed` - 可选的耗时信息（秒）
    pub fn log_message(&self, level: &str, message: &str, elapsed: Option<f64>) -> AppResult<()> {
        let formatted_message = match elapsed {
            Some(elapsed_val) => MessageBuilder::format_message(message, Some(elapsed_val)),
            None => message.to_string(),
        };
        self.log_event(level, &formatted_message)
    }

}
