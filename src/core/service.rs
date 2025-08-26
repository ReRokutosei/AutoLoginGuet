//! 服务层模块，为GUI提供接口
//!
//! 封装了所有的业务逻辑

use crate::core::config::{is_config_complete, load_config, save_config, ConfigData};
use crate::core::crypto::decrypt_password_with_machine_key;
use crate::core::error::{AppError, AppResult};
use crate::core::events::{notify_auto_start_set, notify_config_saved, notify_login_attempted, notify_network_status_checked, DefaultEventHandler, EventBus, EventHandler};
use crate::core::message::MessageCenter;
use crate::core::network::{is_login_successful, NetworkManager, NetworkManagerTrait, NetworkStatus};
use crate::core::{generate_login_error_message, generate_network_status_error_message};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 登录结果处理trait
/// 
/// 为不同的登录结果处理场景定义统一接口
pub trait LoginResultHandler {
    /// 处理登录结果
    /// 
    /// # 参数
    /// * `result` - 登录结果
    /// 
    /// # 返回值
    /// 返回处理后的登录结果
    fn handle(&self, result: LoginResult) -> LoginResult;
}

/// 静默登录结果处理器
pub struct SilentLoginResultHandler {
    message_center: MessageCenter,
    event_bus: EventBus,
}

impl SilentLoginResultHandler {
    pub fn new(message_center: MessageCenter, event_bus: EventBus) -> Self {
        Self {
            message_center,
            event_bus,
        }
    }
}

impl LoginResultHandler for SilentLoginResultHandler {
    fn handle(&self, result: LoginResult) -> LoginResult {
        // 发送登录尝试事件通知
        notify_login_attempted(&self.event_bus, result.success, &result.message, result.elapsed_time);
        
        // 使用MessageCenter处理结果
        self.message_center.handle_login_result_without_event(result)
    }
}

/// 登录结果
#[derive(Debug, Clone)]
pub struct LoginResult {
    /// 登录是否成功
    pub success: bool,
    /// 登录消息
    pub message: String,
    /// 登录耗时（秒）
    pub elapsed_time: f64,
}


/// 认证服务
#[derive(Clone)]
pub struct AuthService {
    network_manager: Arc<Box<dyn NetworkManagerTrait>>,
    message_center: MessageCenter,
    event_bus: EventBus,
    /// 程序启动时间（用于计算从程序启动到完成操作的总时间）
    startup_time: Option<Instant>,
}

impl AuthService {
    /// 创建新的认证服务实例
    pub fn new(config: ConfigData) -> Self {
        Self::new_with_startup_time(config, None)
    }
    
    /// 创建新的认证服务实例，可指定程序启动时间
    pub fn new_with_startup_time(config: ConfigData, startup_time: Option<Instant>) -> Self {
        let network_manager: Box<dyn NetworkManagerTrait> = Box::new(NetworkManager::new(config.network.clone()));
        let event_bus = EventBus::new();
        Arc::new(Mutex::new(Some(Box::new(DefaultEventHandler {}))));
        
        // 注册默认事件处理器到事件总线
        event_bus.register_handler(Box::new(DefaultEventHandler {}));
        
        let message_center = MessageCenter::new(
            Some(config.logging.clone()),
            event_bus.clone(),
        );

        Self {
            network_manager: Arc::new(network_manager),
            message_center,
            event_bus,
            startup_time,
        }
    }

    /// 设置事件处理器
    pub fn set_event_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.event_bus.register_handler(handler);
    }
    
    /// 获取事件总线的引用
    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
    }
    
    /// 检查网络状态
    /// 
    /// `show_notification`: 是否显示通知
    pub async fn check_network_status(&self, show_notification: bool) -> AppResult<NetworkStatus> {
        let start_time = Instant::now();
        let result = self.network_manager.check_network_status().await;
        let elapsed = start_time.elapsed();
        
        match result {
            Ok(status) => {
                let _ = self.message_center.handle_network_status(&status, elapsed.as_secs_f64(), show_notification, false);
                Ok(status)
            }
            Err(e) => {
                self.handle_network_status_error(&e, show_notification, false)
            }
        }
    }
    
    /// 统一的网络状态错误处理函数
    fn handle_network_status_error(&self, e: &AppError, show_notification: bool, should_log: bool) -> AppResult<NetworkStatus> {
        let error_message = generate_network_status_error_message(e);

        if should_log {
            let _ = self.message_center.log_message("ERROR", &format!("网络状态检查失败详情: {}", e), None);
        }
        
        if show_notification {
            let _ = self.message_center.show_notification("", &error_message);

            notify_network_status_checked(&self.event_bus, NetworkStatus::NetworkCheckFailed, "网络状态检查失败");
        }
        
        Ok(NetworkStatus::NetworkCheckFailed)
    }
    
    /// 使用凭据尝试登录
    pub async fn login_with_credentials(&self, username: &str, password: &str, isp: &str) -> AppResult<LoginResult> {
        let start_time = Instant::now();
        
        match self.network_manager.attempt_login_with_credentials(username, password, isp).await {
            Ok(login_text) => {
                let elapsed = start_time.elapsed().as_secs_f64();
                
                if is_login_successful(&login_text) {
                    let result = LoginResult {
                        success: true,
                        message: "登录校园网成功！已接入广域网".to_string(),
                        elapsed_time: elapsed,
                    };
                    
                    Ok(self.message_center.handle_login_result_without_event(result))
                } else {
                    let result = LoginResult {
                        success: false,
                        message: "登录请求失败".to_string(),
                        elapsed_time: elapsed,
                    };
                    
                    Ok(self.message_center.handle_login_result_without_event(result))
                }
            }
            Err(e) => {
                let elapsed = start_time.elapsed().as_secs_f64();

                let gui_message = generate_login_error_message(&e);

                let log_message = format!("登录请求失败: {}", e);

                let _ = self.message_center.log_message("WARNING", &log_message, Some(elapsed));
                
                let result = LoginResult {
                    success: false,
                    message: gui_message,
                    elapsed_time: elapsed,
                };
                
                Ok(result)
            }
        }
    }

    /// 统一处理登录结果的函数
    /// 
    /// 该函数处理登录结果，包括：
    /// 1. 发送登录尝试事件通知
    /// 2. 使用MessageCenter处理结果（日志记录、通知显示等）
    /// 
    /// # 参数
    /// * `result` - 登录结果
    /// 
    /// # 返回值
    /// 返回处理后的登录结果
    pub fn handle_login_result(&self, result: LoginResult) -> LoginResult {
        let handler = SilentLoginResultHandler::new(self.message_center.clone(), self.event_bus.clone());
        handler.handle(result)
    }
    
    /// 静默登录
    pub async fn silent_login(&self, config: ConfigData) -> AppResult<LoginResult> {
        // 如果有启动时间，则使用启动时间为起点；否则使用当前时间为起点
        let method_start_time = Instant::now();
        let start_time = self.startup_time.unwrap_or(method_start_time);

        if config.logging.enable_logging {
            let _ = self.clean_old_logs().map_err(|e| {
                notify_network_status_checked(&self.event_bus, NetworkStatus::NetworkCheckFailed, &format!("清理旧日志失败: {}，将继续执行登录流程", e));
            });
        }

        // 检查网络状态，如果已经登录则直接返回成功消息
        match self.check_network_status(false).await {
            Ok(status) => {
                if status == NetworkStatus::LoggedInAndConnected {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let message = "已登录校园网！网络连接正常";
                    
                    let result = LoginResult {
                        success: true,
                        message: message.to_string(),
                        elapsed_time: elapsed,
                    };
                    
                    // 直接处理结果，但不触发事件（因为check_network_status已经处理了事件）
                    return Ok(self.message_center.handle_login_result_without_event(result));
                }
            }
            Err(e) => {
                notify_network_status_checked(&self.event_bus, NetworkStatus::NetworkCheckFailed, &format!("网络状态检查失败: {}，将继续执行登录流程", e));
            }
        }

        if !is_config_complete(&config) {
            let elapsed = start_time.elapsed().as_secs_f64();
            let message = "配置不完整";
            let result = LoginResult {
                success: false,
                message: message.to_string(),
                elapsed_time: elapsed,
            };
            
            // 直接处理结果
            return Ok(self.message_center.handle_login_result_without_event(result));
        }
        
        let password = match crate::core::crypto::handle_password_decryption_error(
            decrypt_password_with_machine_key(&config.account.encrypted_password),
            &self.event_bus
        ) {
            Ok(pwd) => pwd,
            Err(_) => {
                // 错误已经通过函数`handle_password_decryption_error`处理，这里只需要返回一个失败的LoginResult
                let elapsed = start_time.elapsed().as_secs_f64();
                let result = LoginResult {
                    success: false,
                    message: "密码解密失败，请重新输入密码".to_string(),
                    elapsed_time: elapsed,
                };
                // 直接处理结果
                return Ok(self.message_center.handle_login_result_without_event(result));
            }
        };

        let result = self.login_with_credentials(
            &config.account.username,
            &password,
            &config.account.isp
        ).await;

        match result {
            Ok(login_result) => {
                let elapsed = start_time.elapsed().as_secs_f64();
                let final_result = LoginResult {
                    success: login_result.success,
                    message: login_result.message,
                    elapsed_time: elapsed,
                };
                
                // login_with_credentials已经处理了结果（记录日志和显示通知）
                // 所以这里只需要返回结果，不再重复处理
                Ok(final_result)
            }
            Err(e) => {
                let elapsed = start_time.elapsed().as_secs_f64();

                let gui_message = generate_login_error_message(&e);
                
                let internal_message = format!("登录失败: {:?}", e);
                
                notify_network_status_checked(&self.event_bus, NetworkStatus::NetworkCheckFailed, &internal_message);

                let result = LoginResult {
                    success: false,
                    message: gui_message,
                    elapsed_time: elapsed,
                };
                
                // 直接处理结果
                Ok(self.message_center.handle_login_result_without_event(result))
            }
        }
    }

    /// 加载配置
    pub fn load_config(&self) -> AppResult<ConfigData> {
        load_config()
    }
    
    /// 保存配置
    pub fn save_config(&self, config: &ConfigData) -> AppResult<()> {
        save_config(config)
            .map(|_| {
                notify_config_saved(&self.event_bus, true, "配置保存成功");
            })
            .map_err(|e| {
                notify_config_saved(&self.event_bus, false, &format!("配置保存失败: {:?}", e));
                e
            })
    }
    
    /// 清理旧日志
    pub fn clean_old_logs(&self) -> AppResult<()> {
        self.message_center.clean_old_logs()
            .map_err(|e| AppError::LogError(format!("清理旧日志失败: {}", e)))
    }
    
    /// 设置开机自启
    #[cfg(windows)]
    pub fn set_auto_start(&self, enabled: bool) -> AppResult<()> {
        set_auto_start(enabled)
            .map(|_| {
                notify_auto_start_set(&self.event_bus, enabled, true, if enabled { "开机自启已启用" } else { "开机自启已禁用" });
            })
            .map_err(|e| {
                notify_auto_start_set(&self.event_bus, enabled, false, &format!("开机自启设置失败: {}", e));
                e
            })
    }
}

#[cfg(windows)]
fn set_auto_start(enabled: bool) -> AppResult<()> {
    use std::env;
    use winreg::RegKey;
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (reg_path, app_name) = ("Software\\Microsoft\\Windows\\CurrentVersion\\Run", "AutoLoginGuet");

    if enabled {
        let exe_path = env::current_exe()
            .map_err(|e| AppError::SystemError(format!("获取当前可执行文件路径失败: {}", e)))?;
        
        // 添加 -silent 参数到可执行文件路径
        let exe_path_with_args = format!("\"{}\" -silent", exe_path.to_str().unwrap_or_default());
        
        let reg_key = hkcu.open_subkey_with_flags(reg_path, KEY_SET_VALUE)
            .map_err(|e| AppError::SystemError(format!("无法打开注册表项: {}", e)))?;

        reg_key.set_value(app_name, &exe_path_with_args)
            .map_err(|e| AppError::SystemError(format!("无法设置注册表值: {}", e)))?;
    } else {
        let reg_key = hkcu.open_subkey_with_flags(reg_path, KEY_SET_VALUE)
            .map_err(|e| AppError::SystemError(format!("无法打开注册表项: {}", e)))?;

        // 忽略删除失败的情况（可能键值不存在）
        let _ = reg_key.delete_value(app_name);
    }
    Ok(())
}

/// 验证账号格式
/// 账号应该只包含数字，学生学号通常为10位，教师工号可能较短（3-12位）
pub fn validate_username(username: &str) -> bool {
    // 允许空用户名
    if username.is_empty() {
        return true;
    }
    
    // 检查是否只包含数字
    let filtered: String = username.chars().filter(|c| c.is_ascii_digit()).collect();
    if filtered != *username {
        return false;
    }
    
    // 检查长度是否在3-12位之间
    filtered.len() >= 3 && filtered.len() <= 12
}

/// 验证密码格式
/// 密码必须满足以下要求之一：
/// 1. 长度8-32位，同时包含大小写字母、数字和符号（新规则）
/// 2. 长度8-32位（兼容已有弱密码）
// 新规则是学校校园网官方要求，但该规则可能对部分老生无效
// 因此添加一个仅校验长度的弱密码规则
// 未来如果强制更改为强密码，则可以删除
pub fn validate_password(password: &str) -> bool {
    // 允许空密码（表示不更改密码）
    if password.is_empty() {
        return true;
    }
    
    // 检查长度要求
    if password.len() < 8 || password.len() > 32 {
        return false;
    }
    
    // 检查是否符合新规则（包含大小写字母、数字和符号）
    let mut has_digit = false;
    let mut has_uppercase = false;
    let mut has_lowercase = false;
    let mut has_symbol = false;
    
    for c in password.chars() {
        if c.is_ascii_digit() {
            has_digit = true;
        } else if c.is_ascii_uppercase() {
            has_uppercase = true;
        } else if c.is_ascii_lowercase() {
            has_lowercase = true;
        } else if !c.is_alphanumeric() {
            // 非字母数字字符视为符号
            has_symbol = true;
        }
    }
    
    // 如果符合新规则，直接返回true
    if has_digit && has_uppercase && has_lowercase && has_symbol {
        return true;
    }
    
    // 如果不符合新规则，但长度符合要求，则允许通过（兼容旧规则）
    true
}
