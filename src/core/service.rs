//! 服务层模块，为GUI提供接口
//!
//! 封装了所有的业务逻辑

use crate::core::config::{ConfigData, is_config_complete, load_config, save_config};
use crate::core::crypto::decrypt_password_with_machine_key;
use crate::core::error::{AppError, AppResult};
use crate::core::events::{
    EventBus, EventHandler, notify_auto_start_set, notify_config_saved, notify_login_attempted,
};
use crate::core::flow::FlowService;
use crate::core::message::{CampusNetworkStatus, MessageCenter, WanStatus};
use crate::core::network::{NetworkManager, NetworkManagerTrait};
use std::sync::Arc;
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
        notify_login_attempted(
            &self.event_bus,
            result.success,
            &result.message,
            result.elapsed_time,
        );

        self.message_center
            .handle_login_result_without_event(result)
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
    flow_service: FlowService,
    /// 程序启动时间（计算从程序启动到完成操作的总时间）
    startup_time: Option<Instant>,
}

impl AuthService {
    /// 创建新的认证服务实例
    pub fn new(config: ConfigData) -> Self {
        Self::new_with_startup_time(config, None)
    }

    /// 检查是否需要获取流量信息
    ///
    /// 根据配置判断是否需要调用流量模块
    fn should_get_flow_info(&self) -> bool {
        if let Ok(config) = self.load_config()
            && config.account.isp.is_empty()
        {
            // 检查消息配置中是否包含%4占位符
            return config.message.notify_text.contains("%4")
                || config.message.gui_text.contains("%4")
                || config.message.log_text.contains("%4");
        }
        false
    }

    /// 检查是否需要检查广域网状态
    fn should_check_wan(&self) -> bool {
        if let Ok(config) = self.load_config() {
            // 检查消息配置中是否包含%2占位符
            return config.message.notify_text.contains("%2")
                || config.message.gui_text.contains("%2")
                || config.message.log_text.contains("%2");
        }
        false
    }

    /// 创建新的认证服务实例，可指定程序启动时间
    pub fn new_with_startup_time(config: ConfigData, startup_time: Option<Instant>) -> Self {
        let network_manager: Box<dyn NetworkManagerTrait> =
            Box::new(NetworkManager::new(config.network.clone()));
        let event_bus = EventBus::new();

        let message_center = MessageCenter::new(Some(config.clone()), event_bus.clone());
        let flow_service = FlowService::new();

        Self {
            network_manager: Arc::new(network_manager),
            message_center,
            event_bus,
            flow_service,
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

    /// 获取消息中心的引用
    pub fn get_message_center(&self) -> &MessageCenter {
        &self.message_center
    }

    /// 检查网络状态
    ///
    /// `show_notification`: 是否显示通知
    pub async fn check_network_status(
        &self,
        show_notification: bool,
    ) -> AppResult<(CampusNetworkStatus, WanStatus)> {
        let start_time = Instant::now();

        // 检查校园网状态
        let campus_result = self.network_manager.check_campus_network().await;

        // 根据配置决定是否检查广域网状态
        let wan_status = if self.should_check_wan() {
            self.network_manager.check_wan_network().await
        } else {
            // 如果不需要检查广域网，返回默认状态
            WanStatus::CheckFailed
        };

        let elapsed = start_time.elapsed().as_secs_f64();

        // 处理校园网检查结果
        let campus_status = campus_result.unwrap_or({
            // 校园网检查失败，返回错误状态
            CampusNetworkStatus::NotLoggedIn
        });

        // 获取流量信息（如果需要）
        let flow_info = if self.should_get_flow_info() {
            // 加载配置以获取账号密码
            if let Ok(config) = self.load_config() {
                // 只有当配置完整时才获取流量信息
                if !config.account.username.is_empty()
                    && !config.account.encrypted_password.is_empty()
                {
                    match self
                        .flow_service
                        .get_user_flow_info(
                            &config.account.username,
                            &decrypt_password_with_machine_key(&config.account.encrypted_password)?,
                        )
                        .await
                    {
                        Ok(flow) => Some(flow.left_flow),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 使用MessageCenter处理网络状态结果
        let _ = self.message_center.handle_network_status(
            campus_status.clone(),
            wan_status.clone(),
            elapsed,
            show_notification,
            false,
            flow_info,
        );

        Ok((campus_status, wan_status))
    }

    /// 使用凭据尝试登录
    pub async fn login_with_credentials(
        &self,
        username: &str,
        password: &str,
        isp: &str,
    ) -> AppResult<LoginResult> {
        let start_time = Instant::now();

        match self
            .network_manager
            .attempt_login_with_credentials(username, password, isp)
            .await
        {
            Ok(login_result) => {
                let elapsed = start_time.elapsed().as_secs_f64();

                let wan_status = if self.should_check_wan() {
                    self.network_manager.check_wan_network().await
                } else {
                    WanStatus::CheckFailed
                };

                let flow_info = if self.should_get_flow_info() {
                    match self
                        .flow_service
                        .get_user_flow_info(username, password)
                        .await
                    {
                        Ok(flow) => Some(flow.left_flow),
                        Err(_) => None,
                    }
                } else {
                    None
                };

                let message = self.message_center.handle_login_result(
                    login_result.campus_status.clone(),
                    wan_status,
                    elapsed,
                    login_result.success,
                    flow_info,
                );

                let result = LoginResult {
                    success: login_result.success,
                    message,
                    elapsed_time: elapsed,
                };

                Ok(result)
            }
            Err(e) => {
                let elapsed = start_time.elapsed().as_secs_f64();

                let gui_message = format!("登录失败: {}", e);

                let log_message = format!("登录请求失败: {}", e);

                let _ = self.message_center.log_event("WARNING", &log_message);

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
        let handler =
            SilentLoginResultHandler::new(self.message_center.clone(), self.event_bus.clone());
        handler.handle(result)
    }

    /// 静默登录
    pub async fn silent_login(&self, config: ConfigData) -> AppResult<LoginResult> {
        // 如果有启动时间，则使用启动时间为起点；否则使用当前时间为起点
        let method_start_time = Instant::now();
        let start_time = self.startup_time.unwrap_or(method_start_time);

        if config.logging.enable_logging {
            let _ = self.clean_old_logs().map_err(|e| {
                // 使用新的消息系统处理错误通知
                let _ = self
                    .message_center
                    .show_notification("", &format!("清理旧日志失败: {}，将继续执行登录流程", e));
            });
        }

        // 检查网络状态，如果已经登录则直接返回成功消息
        match self.check_network_status(false).await {
            Ok((campus_status, wan_status)) => {
                if campus_status == CampusNetworkStatus::AlreadyLoggedIn {
                    let elapsed = start_time.elapsed().as_secs_f64();

                    let flow_info = if self.should_get_flow_info() {
                        if let Ok(config) = self.load_config() {
                            match self
                                .flow_service
                                .get_user_flow_info(
                                    &config.account.username,
                                    &decrypt_password_with_machine_key(
                                        &config.account.encrypted_password,
                                    )?,
                                )
                                .await
                            {
                                Ok(flow) => Some(flow.left_flow),
                                Err(_) => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let message = self.message_center.handle_network_status(
                        campus_status,
                        wan_status,
                        elapsed,
                        true,
                        true,
                        flow_info,
                    );

                    let result = LoginResult {
                        success: true,
                        message,
                        elapsed_time: elapsed,
                    };

                    return Ok(result);
                }
            }
            Err(e) => {
                let _ = self
                    .message_center
                    .show_notification("", &format!("网络状态检查失败: {}，将继续执行登录流程", e));
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

            let _ = self
                .message_center
                .log_event("ERROR", &format!("{} 用时{:.2}秒", message, elapsed));
            let _ = self.message_center.show_notification("", message);

            return Ok(result);
        }

        let password = match crate::core::crypto::handle_password_decryption_error(
            decrypt_password_with_machine_key(&config.account.encrypted_password),
            &self.event_bus,
        ) {
            Ok(pwd) => pwd,
            Err(_) => {
                // 错误已经通过`handle_password_decryption_error`处理，这里只需要返回一个失败的LoginResult
                let elapsed = start_time.elapsed().as_secs_f64();
                let message = "密码解密失败，请重新输入密码";

                let result = LoginResult {
                    success: false,
                    message: message.to_string(),
                    elapsed_time: elapsed,
                };

                // 记录日志和显示通知
                let _ = self
                    .message_center
                    .log_event("ERROR", &format!("{} 用时{:.2}秒", message, elapsed));
                let _ = self.message_center.show_notification("", message);

                return Ok(result);
            }
        };

        let result = self
            .login_with_credentials(&config.account.username, &password, &config.account.isp)
            .await;

        match result {
            Ok(login_result) => {
                let elapsed = start_time.elapsed().as_secs_f64();
                let final_result = LoginResult {
                    success: login_result.success,
                    message: login_result.message,
                    elapsed_time: elapsed,
                };
                // login_with_credentials已经处理了结果
                // 这里只需要返回结果，不再重复处理
                Ok(final_result)
            }
            Err(e) => {
                let elapsed = start_time.elapsed().as_secs_f64();

                let gui_message = format!("登录失败: {}", e);

                let _ = self
                    .message_center
                    .log_event("ERROR", &format!("登录失败: {} 用时{:.2}秒", e, elapsed));
                let _ = self.message_center.show_notification("", &gui_message);

                let result = LoginResult {
                    success: false,
                    message: gui_message,
                    elapsed_time: elapsed,
                };

                Ok(result)
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
        self.message_center
            .clean_old_logs()
            .map_err(|e| AppError::LogError(format!("清理旧日志失败: {}", e)))
    }

    /// 设置开机自启
    #[cfg(windows)]
    pub fn set_auto_start(&self, enabled: bool) -> AppResult<()> {
        set_auto_start(enabled)
            .map(|_| {
                notify_auto_start_set(
                    &self.event_bus,
                    enabled,
                    true,
                    if enabled {
                        "开机自启已启用"
                    } else {
                        "开机自启已禁用"
                    },
                );
            })
            .map_err(|e| {
                notify_auto_start_set(
                    &self.event_bus,
                    enabled,
                    false,
                    &format!("开机自启设置失败: {}", e),
                );
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
    let (reg_path, app_name) = (
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        "AutoLoginGuet",
    );

    if enabled {
        let exe_path = env::current_exe()
            .map_err(|e| AppError::SystemError(format!("获取当前可执行文件路径失败: {}", e)))?;

        // 添加 -silent 参数到可执行文件路径
        let exe_path_with_args = format!("\"{}\" -silent", exe_path.to_str().unwrap_or_default());

        let reg_key = hkcu
            .open_subkey_with_flags(reg_path, KEY_SET_VALUE)
            .map_err(|e| AppError::SystemError(format!("无法打开注册表项: {}", e)))?;

        reg_key
            .set_value(app_name, &exe_path_with_args)
            .map_err(|e| AppError::SystemError(format!("无法设置注册表值: {}", e)))?;
    } else {
        let reg_key = hkcu
            .open_subkey_with_flags(reg_path, KEY_SET_VALUE)
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
