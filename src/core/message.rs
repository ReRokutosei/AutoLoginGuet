//! 消息处理中心模块
//!
//! 集中管理所有消息的生成、日志记录和通知显示

use crate::core::config::{ConfigData, MessageConfig};
use crate::core::error::{AppError, AppResult};
use crate::core::events::EventBus;
use crate::core::service::LoginResult;
use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use notify_rust::Notification;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// 校园网状态枚举
#[derive(Debug, Clone, PartialEq)]
pub enum CampusNetworkStatus {
    /// 已登录校园网（登录状态未失效）
    AlreadyLoggedIn,
    /// 未登录校园网
    NotLoggedIn,
    /// 登录校园网成功（从未登录状态到登录状态的转变）
    LoginSuccess,
}

impl CampusNetworkStatus {
    /// 转换为消息文本
    pub fn to_message(&self) -> String {
        match self {
            CampusNetworkStatus::AlreadyLoggedIn => "已登录校园网".to_string(),
            CampusNetworkStatus::NotLoggedIn => "未登录校园网".to_string(),
            CampusNetworkStatus::LoginSuccess => "登录校园网成功！".to_string(),
        }
    }
}

/// 广域网状态枚举
#[derive(Debug, Clone, PartialEq)]
pub enum WanStatus {
    /// 已接入广域网
    Connected,
    /// 无法访问广域网
    Disconnected,
    /// 广域网检查失败
    CheckFailed,
}

impl WanStatus {
    /// 转换为消息文本
    pub fn to_message(&self) -> String {
        match self {
            WanStatus::Connected => "已接入广域网".to_string(),
            WanStatus::Disconnected => "无法访问广域网".to_string(),
            WanStatus::CheckFailed => "广域网检查失败".to_string(),
        }
    }
}

/// 消息构建结果
#[derive(Debug, Clone)]
pub struct MessageResult {
    /// 通知消息
    pub notify_message: String,
    /// GUI消息
    pub gui_message: String,
    /// 日志消息
    pub log_message: String,
}

/// 消息构建器（构建者模式）
pub struct MessageBuilder {
    campus_status: Option<CampusNetworkStatus>,
    wan_status: Option<WanStatus>,
    elapsed_time: Option<f64>,
    flow_info: Option<String>,
    isp: String,
}

impl MessageBuilder {
    /// 创建新的消息构建器
    pub fn new(isp: String) -> Self {
        Self {
            campus_status: None,
            wan_status: None,
            elapsed_time: None,
            flow_info: None,
            isp,
        }
    }

    /// 设置校园网状态
    pub fn with_campus_status(mut self, status: CampusNetworkStatus) -> Self {
        self.campus_status = Some(status);
        self
    }

    /// 设置广域网状态
    pub fn with_wan_status(mut self, status: WanStatus) -> Self {
        self.wan_status = Some(status);
        self
    }

    /// 设置耗时信息
    pub fn with_elapsed_time(mut self, elapsed: f64) -> Self {
        self.elapsed_time = Some(elapsed);
        self
    }

    /// 设置流量信息（MB）
    pub fn with_flow_info(mut self, flow_mb: f64) -> Self {
        self.flow_info = Some(Self::format_flow_info(flow_mb));
        self
    }

    /// 格式化流量信息
    fn format_flow_info(flow_mb: f64) -> String {
        if flow_mb == 0.0 {
            "流量耗尽，限速不限量生效".to_string()
        } else if flow_mb >= 1024.0 {
            format!("剩余流量{:.2}GB", flow_mb / 1024.0)
        } else {
            format!("剩余流量{:.2}MB", flow_mb)
        }
    }

    /// 构建消息结果
    pub fn build(self, config: &MessageConfig) -> MessageResult {
        let campus_message = self
            .campus_status
            .as_ref()
            .map(|s| s.to_message())
            .unwrap_or_default();

        // 只有在需要显示广域网信息时才显示广域网状态消息
        let wan_message = if matches!(self.wan_status, Some(WanStatus::CheckFailed)) {
            String::new()
        } else {
            self.wan_status
                .as_ref()
                .map(|s| s.to_message())
                .unwrap_or_default()
        };

        let time_message = self
            .elapsed_time
            .map(|t| format!("用时{:.2}秒", t))
            .unwrap_or_default();

        let flow_message = if self.isp.is_empty() {
            // 校园网运营商，显示流量信息
            self.flow_info.unwrap_or_default()
        } else {
            // 非校园网运营商，过滤流量信息
            String::new()
        };

        // 处理占位符替换
        let notify_message = Self::replace_placeholders(
            &config.notify_text,
            &campus_message,
            &wan_message,
            &time_message,
            &flow_message,
            true,
        );

        let gui_message = Self::replace_placeholders(
            &config.gui_text,
            &campus_message,
            &wan_message,
            &time_message,
            &flow_message,
            true,
        );

        let log_message = Self::replace_placeholders(
            &config.log_text,
            &campus_message,
            &wan_message,
            &time_message,
            &flow_message,
            false,
        );

        MessageResult {
            notify_message,
            gui_message,
            log_message,
        }
    }

    /// 替换占位符
    fn replace_placeholders(
        template: &str,
        campus: &str,
        wan: &str,
        time: &str,
        flow: &str,
        allow_newlines: bool,
    ) -> String {
        let mut result = template.to_string();

        // 替换占位符
        result = result.replace("%1", campus);
        result = result.replace("%2", wan);
        result = result.replace("%3", time);
        result = result.replace("%4", flow);

        if !allow_newlines {
            result = result.replace('\n', " ");
        }

        while result.contains("  ") {
            result = result.replace("  ", " ");
        }
        result = result.trim().to_string();

        if wan.is_empty() {
            result = result.replace("  ", " ").trim().to_string();
        }

        result
    }
}

impl Default for MessageCenter {
    /// 创建默认的消息处理中心实例（无日志配置）
    fn default() -> Self {
        Self {
            config: None,
            write_mutex: Arc::new(Mutex::new(())),
            _event_bus: EventBus::new(),
        }
    }
}

/// 消息处理中心
#[derive(Clone)]
pub struct MessageCenter {
    config: Option<ConfigData>,
    /// 用于同步日志写入操作的互斥锁
    write_mutex: Arc<Mutex<()>>,
    _event_bus: EventBus,
}

impl MessageCenter {
    /// 创建新的消息处理中心实例
    pub fn new(config: Option<ConfigData>, event_bus: EventBus) -> Self {
        Self {
            config,
            write_mutex: Arc::new(Mutex::new(())),
            _event_bus: event_bus,
        }
    }

    /// 处理网络状态检查结果
    pub fn handle_network_status(
        &self,
        campus_status: CampusNetworkStatus,
        wan_status: WanStatus,
        elapsed: f64,
        show_notification: bool,
        should_log: bool,
        flow_info: Option<f64>,
    ) -> String {
        // 如果没有配置，使用默认消息
        let Some(config) = self.config.as_ref() else {
            return format!("{} {}", campus_status.to_message(), wan_status.to_message());
        };

        let isp = config.account.isp.clone();

        let mut builder = MessageBuilder::new(isp)
            .with_campus_status(campus_status.clone())
            .with_wan_status(wan_status.clone())
            .with_elapsed_time(elapsed);

        // 如果有流量信息，添加到构建器
        if let Some(flow) = flow_info {
            builder = builder.with_flow_info(flow);
        }

        let result = builder.build(&config.message);

        if should_log {
            let _ = self.log_event("INFO", &result.log_message);
        }

        if show_notification {
            let _ = self.show_notification("", &result.notify_message);
        }

        result.gui_message
    }

    /// 处理登录结果
    pub fn handle_login_result(
        &self,
        campus_status: CampusNetworkStatus,
        wan_status: WanStatus,
        elapsed: f64,
        success: bool,
        flow_info: Option<f64>,
    ) -> String {
        // 如果没有配置，使用默认消息
        let Some(config) = self.config.as_ref() else {
            return format!("{} {}", campus_status.to_message(), wan_status.to_message());
        };

        let isp = config.account.isp.clone();

        let mut builder = MessageBuilder::new(isp)
            .with_campus_status(campus_status)
            .with_wan_status(wan_status)
            .with_elapsed_time(elapsed);

        // 如果有流量信息，则添加到构建器
        if let Some(flow) = flow_info {
            builder = builder.with_flow_info(flow);
        }

        let result = builder.build(&config.message);

        let log_level = if success { "INFO" } else { "ERROR" };
        let _ = self.log_event(log_level, &result.log_message);

        let _ = self.show_notification("", &result.notify_message);

        result.gui_message
    }

    /// 处理登录结果但不触发事件（用于静默模式）
    pub fn handle_login_result_without_event(&self, result: LoginResult) -> LoginResult {
        // 如果没有配置，直接返回结果
        let Some(config) = self.config.as_ref() else {
            return result;
        };

        let isp = config.account.isp.clone();

        // 构建消息但不触发事件
        let builder = MessageBuilder::new(isp);

        let message_result = builder.build(&config.message);

        // 只记录日志，不显示通知
        let log_level = if result.success { "INFO" } else { "ERROR" };
        let _ = self.log_event(log_level, &message_result.log_message);

        result
    }

    /// 显示通知
    pub fn show_notification(&self, _title: &str, message: &str) -> AppResult<()> {
        Notification::new()
            .summary(message)
            .show()
            .map_err(|e| AppError::NotificationError(e.to_string()))
    }

    /// 记录日志事件
    pub fn log_event(&self, level: &str, message: &str) -> AppResult<()> {
        let Some(config) = self.config.as_ref() else {
            return Ok(());
        };

        if !config.logging.enable_logging {
            return Ok(());
        }

        let _guard = self.write_mutex.lock().unwrap();

        let log_file_path = config.logging.log_file_path.as_str();
        let log_entry = format!(
            "[{}][{}] {}\n",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            level,
            message
        );

        if let Some(parent) = Path::new(log_file_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::SystemError(format!("无法创建日志目录: {}", e)))?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .map_err(|e| {
                AppError::SystemError(format!("无法打开日志文件 '{}': {}", log_file_path, e))
            })?;

        file.write_all(log_entry.as_bytes())
            .map_err(|e| AppError::SystemError(format!("无法写入日志文件: {}", e)))?;

        Ok(())
    }

    /// 读取日志
    pub fn read_logs(&self) -> AppResult<String> {
        let log_file_path = match self.config.as_ref() {
            Some(c) if c.logging.enable_logging => &c.logging.log_file_path,
            _ => return Ok(String::new()),
        };

        if !Path::new(log_file_path).exists() {
            return Ok(String::new());
        }

        let file = File::open(log_file_path).map_err(|e| {
            AppError::SystemError(format!("无法打开日志文件 '{}': {}", log_file_path, e))
        })?;

        let reader = BufReader::new(file);
        let mut logs = String::new();

        for line in reader.lines() {
            logs.push_str(
                &line.map_err(|e| AppError::SystemError(format!("读取日志行失败: {}", e)))?,
            );
            logs.push('\n');
        }

        Ok(logs)
    }

    /// 清理旧日志
    pub fn clean_old_logs(&self) -> AppResult<()> {
        let Some(config) = self.config.as_ref() else {
            return Ok(());
        };

        if !config.logging.enable_logging {
            return Ok(());
        }

        let _guard = self.write_mutex.lock().unwrap();

        let log_file_path = config.logging.log_file_path.as_str();

        if !Path::new(log_file_path).exists() {
            return Ok(());
        }

        let cutoff_date = Local::now() - Duration::days(config.logging.info_log_retention_days);
        let temp_file_path = format!("{}.tmp", log_file_path);

        {
            let input_file = File::open(log_file_path).map_err(|e| {
                AppError::SystemError(format!("无法打开日志文件 '{}': {}", log_file_path, e))
            })?;
            let output_file = File::create(&temp_file_path).map_err(|e| {
                AppError::SystemError(format!("无法创建临时文件 '{}': {}", temp_file_path, e))
            })?;

            let reader = BufReader::new(input_file);
            let mut writer = io::BufWriter::new(output_file);

            for line in reader.lines() {
                let line =
                    line.map_err(|e| AppError::SystemError(format!("读取日志行失败: {}", e)))?;

                if let Some(date_part) = line.get(1..20) {
                    if let Ok(log_date) =
                        NaiveDateTime::parse_from_str(date_part, "%Y-%m-%d %H:%M:%S")
                    {
                        let log_date = Local.from_local_datetime(&log_date).unwrap();
                        if log_date >= cutoff_date {
                            writeln!(writer, "{}", line).map_err(|e| {
                                AppError::SystemError(format!("写入临时文件失败: {}", e))
                            })?;
                        }
                    } else {
                        writeln!(writer, "{}", line).map_err(|e| {
                            AppError::SystemError(format!("写入临时文件失败: {}", e))
                        })?;
                    }
                } else {
                    writeln!(writer, "{}", line)
                        .map_err(|e| AppError::SystemError(format!("写入临时文件失败: {}", e)))?;
                }
            }

            writer
                .flush()
                .map_err(|e| AppError::SystemError(format!("刷新临时文件失败: {}", e)))?;
        }

        fs::rename(&temp_file_path, log_file_path)
            .map_err(|e| AppError::SystemError(format!("替换日志文件失败: {}", e)))?;

        let _ = fs::remove_file(&temp_file_path);

        Ok(())
    }
}
