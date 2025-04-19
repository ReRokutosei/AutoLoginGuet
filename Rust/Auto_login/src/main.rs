use chrono::Local;
use notify_rust::Notification;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use rand::Rng;
use std::thread;
use std::io::Write;

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Config {
    network: NetworkConfig,
    notification: NotificationConfig,
    logging: LoggingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            network: NetworkConfig::default(),
            notification: NotificationConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct NetworkConfig {
    login_ip: String,
    sign_parameter: String,
    result_return: String,
    signed_in_title: String,
    not_sign_in_title: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            login_ip: String::new(),
            sign_parameter: String::new(),
            result_return: String::new(),
            signed_in_title: String::new(),
            not_sign_in_title: String::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct NotificationConfig {}

impl Default for NotificationConfig {
    fn default() -> Self {
        NotificationConfig {}
    }
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    enable_logging: bool,
    log_file_path: String,
    info_log_retention_days: i64,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            enable_logging: false,
            log_file_path: String::new(),
            info_log_retention_days: 7,
        }
    }
}

struct LogManager {
    config: LoggingConfig,
}

impl LogManager {
    fn new(config: LoggingConfig) -> Self {
        LogManager { config }
    }

    fn log_event(&self, level: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enable_logging {
            return Ok(());
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_entry = format!("{} - {} - {}\r\n", timestamp, level, message);
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_file_path)?;
        file.write_all(log_entry.as_bytes())?;
        Ok(())
    }

    fn clean_old_logs(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enable_logging {
            return Ok(());
        }

        if Path::new(&self.config.log_file_path).exists() {
            let content = fs::read_to_string(&self.config.log_file_path)?;
            let cutoff_date = Local::now()
                .checked_sub_signed(chrono::Duration::days(self.config.info_log_retention_days))
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            
            let mut new_lines = Vec::new();
            for line in content.lines() {
                if let Ok(log_date) = chrono::NaiveDateTime::parse_from_str(&line[..19], "%Y-%m-%d %H:%M:%S") {
                    if line.contains("INFO") && log_date < chrono::NaiveDateTime::parse_from_str(&cutoff_date, "%Y-%m-%d %H:%M:%S")? {
                        continue;
                    }
                }
                new_lines.push(format!("{}\r\n", line));
            }
            
            // 写入新内容，不需要额外的换行符
            fs::write(&self.config.log_file_path, new_lines.join(""))?;
        }
        Ok(())
    }
}

fn show_notification(_config: &NotificationConfig, title: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .summary(title)
        .body(message)
        .show()?;
    Ok(())
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let config_path = exe_dir.join("config.yaml");
    
    let contents = fs::read_to_string(&config_path)?;
    let mut config: Config = serde_yaml::from_str(&contents)?;
    
    // 处理日志路径
    if config.logging.enable_logging {
        let log_path = Path::new(&config.logging.log_file_path);
        if log_path.to_str().unwrap().starts_with("./") || log_path.to_str().unwrap().starts_with("../") {
            config.logging.log_file_path = exe_dir.join(log_path.strip_prefix("./").unwrap_or(log_path))
                .to_str().unwrap().to_string();
        }
    }
    
    Ok(config)
}

// 修改main函数，添加全局错误处理
fn main() {
    if let Err(e) = run() {
        // 这里可以尝试记录错误，即使日志系统可能已经失败
        eprintln!("发生错误: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let config = load_config()?;
    let log_manager = LogManager::new(config.logging);
    
    log_manager.clean_old_logs()?;
    
    let delay = rand::thread_rng().gen_range(0..5000);
    thread::sleep(Duration::from_millis(delay));

    let client = Client::new();
    
    // 先检查登录状态
    match client.get(&config.network.login_ip)
        .timeout(Duration::from_secs(10))
        .send()
    {
        Ok(response) => {
            let text = response.text()?;
            let elapsed = start_time.elapsed().as_secs_f64();

            if text.contains(&config.network.signed_in_title) {
                let log_message = format!("Device already logged in. Elapsed time: {:.2} seconds.", elapsed);
                let status = format!("该设备已经登录，本次用时 {:.2} 秒", elapsed);
                log_manager.log_event("INFO", &log_message)?;
                show_notification(&config.notification, "校园网状态", &status)?;
            } else if text.contains(&config.network.not_sign_in_title) {
                // 尝试登录
                match client.get(&config.network.sign_parameter)
                    .timeout(Duration::from_secs(10))
                    .send()
                {
                    Ok(login_response) => {
                        let login_text = login_response.text()?;
                        let elapsed = start_time.elapsed().as_secs_f64();
                        
                        if login_text.contains(&config.network.result_return) {
                            let log_message = format!("Login successful. Elapsed time: {:.2} seconds.", elapsed);
                            let status = format!("登录成功，本次用时 {:.2} 秒", elapsed);
                            log_manager.log_event("INFO", &log_message)?;
                            show_notification(&config.notification, "校园网状态", &status)?;
                        } else {
                            let log_message = format!("Login failed. Elapsed time: {:.2} seconds.", elapsed);
                            let status = format!("登录失败，本次用时 {:.2} 秒", elapsed);
                            log_manager.log_event("WARNING", &log_message)?;
                            show_notification(&config.notification, "校园网状态", &status)?;
                        }
                    }
                    Err(e) => {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let log_message = format!("Login request failed: {}. Elapsed time: {:.2} seconds.", e, elapsed);
                        let status = format!("登录请求失败，本次用时 {:.2} 秒", elapsed);
                        log_manager.log_event("ERROR", &log_message)?;
                        show_notification(&config.notification, "校园网状态", &status)?;
                    }
                }
            } else {
                let log_message = format!("Not connected to campus network. Elapsed time: {:.2} seconds.", elapsed);
                let status = format!("未连接到校园网，本次用时 {:.2} 秒", elapsed);
                log_manager.log_event("WARNING", &log_message)?;
                show_notification(&config.notification, "校园网状态", &status)?;
            }
        }
        Err(e) => {
            let elapsed = start_time.elapsed().as_secs_f64();
            let log_message = format!("Initial network check failed: {}. Elapsed time: {:.2} seconds.", e, elapsed);
            let status = format!("网络检查失败，本次用时 {:.2} 秒", elapsed);
            log_manager.log_event("ERROR", &log_message)?;
            show_notification(&config.notification, "校园网状态", &status)?;
        }
    }

    Ok(())
}