//! 网络管理模块
//!
//! 负责处理应用程序的所有网络相关功能

use reqwest::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use urlencoding::encode;
use crate::core::error::{AppError, AppResult, NetworkError};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::core::config::normalize_isp;
use async_trait::async_trait;

const EXTERNAL_SITES: [&str; 5] = [
    "https://www.baidu.com",
    "https://www.qq.com",
    "https://www.sina.com.cn",
    "https://www.alibaba.com",
    "https://www.taobao.com",
];

// 限制响应体大小为10MB（对于校园网认证足够大，同时防止极端情况）
const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36";

/// 网络状态枚举
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkStatus {
    /// 已登录校园网且能访问广域网
    LoggedInAndConnected,
    /// 未登录校园网但能访问广域网
    NotLoggedInButConnected,
    /// 网络检查失败
    NetworkCheckFailed,
    /// 无法访问校园网且无法访问广域网
    NoNetwork,
    /// 已登录校园网但不能访问广域网
    LoggedInButNoWan,
}

impl NetworkStatus {
    /// 转换为消息文本
    /// 
    /// # 返回值
    /// 返回格式化的消息文本
    pub fn to_message(&self) -> String {
        match self {
            NetworkStatus::LoggedInAndConnected => {
                "已登录校园网！网络连接正常".to_string()
            }
            NetworkStatus::NotLoggedInButConnected => {
                "未登录校园网，但已连接其他网络".to_string()
            }
            NetworkStatus::NetworkCheckFailed => {
                "网络检查失败".to_string()
            }
            NetworkStatus::NoNetwork => {
                "未登录校园网".to_string()
            }
            NetworkStatus::LoggedInButNoWan => {
                "已登录校园网，但无法访问广域网".to_string()
            }
        }
    }
}

/// 网络管理trait，定义网络操作接口
#[async_trait]
pub trait NetworkManagerTrait: Send + Sync {
    /// 检查网络状态
    async fn check_network_status(&self) -> AppResult<NetworkStatus>;
    
    /// 检查网络状态（带选项）
    /// 
    /// # 参数
    /// * `_should_log` - 是否应该记录日志
    async fn check_network_status_with_options(&self, _should_log: bool) -> AppResult<NetworkStatus> {
        // 默认实现，忽略should_log参数
        self.check_network_status().await
    }
    
    /// 使用凭据尝试登录
    async fn attempt_login_with_credentials(&self, username: &str, password: &str, isp: &str) 
        -> AppResult<String>;
    
    /// 克隆网络管理器
    fn clone_box(&self) -> Box<dyn NetworkManagerTrait>;
}

/// 网络配置结构体
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct NetworkConfig {
    /// 登录IP地址
    pub login_ip: String,
    /// 登录成功返回结果标识
    pub result_return: String,
    /// 已登录页面标题标识
    pub signed_in_title: String,
    /// 未登录页面标题标识
    pub not_sign_in_title: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            login_ip: "http://10.0.1.5/".to_string(),
            result_return: "\"result\":1".to_string(),
            signed_in_title: "注销页".to_string(),
            not_sign_in_title: "上网登录页".to_string(),
        }
    }
}

/// 网络管理器
#[derive(Debug, Clone)]
pub struct NetworkManager {
    client: Client,
    config: NetworkConfig,
}

impl NetworkManager {
    /// 创建新的网络管理器实例
    pub fn new(config: NetworkConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
            
        Self { client, config }
    }
    
    fn get_client(&self) -> &Client {
        &self.client
    }

    /// 检查校园网状态
    pub async fn check_campus_network_status(&self) -> AppResult<NetworkStatus> {
        let response = self.get_client()
            .get(&self.config.login_ip)
            .send()
            .await
            .map_err(|e| AppError::NetworkError { source: crate::core::error::map_reqwest_error(e) })?;
            
        let status = response.status();
        
        // 对于校园网状态检查，只需要检查标题或者少量内容即可判断状态
        let text = response.text().await.map_err(|e| AppError::NetworkError { source: NetworkError::Other(e.to_string()) })?;
        let truncated_text = if text.len() > 4096 {
            text[..4096].to_string()
        } else {
            text
        };
        
        if status.is_redirection() {
            Ok(NetworkStatus::NotLoggedInButConnected)
        } else if truncated_text.contains(&self.config.signed_in_title) {
            Ok(NetworkStatus::LoggedInAndConnected)
        } else if truncated_text.contains(&self.config.not_sign_in_title) {
            Ok(NetworkStatus::NotLoggedInButConnected)
        } else {
            // 当无法识别校园网页面时，视为未登录且无网络连接
            Ok(NetworkStatus::NoNetwork)
        }
    }

    /// 检查外部网络连接状态
    async fn check_external_connectivity(&self) -> bool {
        let cancelled = Arc::new(AtomicBool::new(false));
        
        let mut futures = Vec::new();
        
        for &site in &EXTERNAL_SITES {
            let client = self.get_client().clone();
            let cancelled_clone = cancelled.clone();
            let login_ip = self.config.login_ip.clone();
            let not_sign_in_title = self.config.not_sign_in_title.clone();
            
            let future = async move {
                if cancelled_clone.load(Ordering::Relaxed) {
                    return false;
                }
                
                let result = tokio::time::timeout(
                    Duration::from_secs(5),
                    client.get(site).send()
                ).await;
                
                match result {
                    Ok(Ok(response)) => {
                        let status = response.status();
                        
                        // 检查是否被重定向到校园网登录页面
                        if let Some(url) = response.url().host_str() {
                            if login_ip.contains(url) {
                                // 被重定向到校园网登录页面，说明未连接外网
                                return false;
                            }
                        }
                        
                        // 检查响应内容是否包含登录页面标题
                        let text_result = tokio::time::timeout(
                            Duration::from_secs(3),
                            response.text()
                        ).await;
                        
                        if let Ok(Ok(text)) = text_result {
                            if text.contains(&not_sign_in_title) {
                                // 响应内容是登录页面，说明未连接外网
                                return false;
                            }
                        }
                        
                        status.is_success()
                    },
                    _ => false,
                }
            };
            
            futures.push(future);
        }
        
        for future in futures {
            if future.await {
                // 一旦有一个网站可以访问，就取消其他请求并返回true
                cancelled.store(true, Ordering::Relaxed);
                return true;
            }
        }
        
        false
    }
    
    /// 检查外部网络连接状态并返回NetworkStatus
    async fn check_external_connectivity_status(&self) -> NetworkStatus {
        let result = self.check_external_connectivity().await;
        if result {
            // 当能访问外网时，视为NotLoggedInButConnected
            NetworkStatus::NotLoggedInButConnected
        } else {
            // 无法访问外网时，视为NoNetwork
            NetworkStatus::NoNetwork
        }
    }
    
    /// 检查网络状态（公共接口）
    pub async fn check_network_status(&self) -> AppResult<NetworkStatus> {
        // 首先检查是否能访问校园网
        let campus_status = self.check_campus_network_status().await;
        
        match campus_status {
            Ok(NetworkStatus::LoggedInAndConnected) => {
                // 如果已登录校园网，直接返回该状态
                Ok(NetworkStatus::LoggedInAndConnected)
            }
            Ok(NetworkStatus::NotLoggedInButConnected) => {
                // 未登录但可以访问校园网登录页面，需要进一步检查外网连接
                let external_status = self.check_external_connectivity_status().await;
                Ok(external_status)
            }
            Err(_) => {
                // 校园网检查失败，检查是否能访问外网
                let external_status = self.check_external_connectivity_status().await;
                Ok(external_status)
            }
            _ => {
                let external_status = self.check_external_connectivity_status().await;
                Ok(external_status)
            }
        }
    }

    /// 使用凭据尝试登录
    pub async fn attempt_login_with_credentials(&self, username: &str, password: &str, isp: &str) -> AppResult<String> {
        self.try_drcom_login(username, password, &normalize_isp(isp)).await
    }
    
    async fn try_drcom_login(&self, username: &str, password: &str, isp: &str) -> AppResult<String> {
        // URL编码密码
        let encoded_password = encode(password);
        
        // 构造（学号+运营商）
        // - 校园网:   学号 (不带任何后缀)
        // - 中国移动: 学号@cmcc
        // - 中国联通: 学号@unicom
        // - 中国电信: 学号@telecom
        let full_username = if isp.is_empty() {
            username.to_string()
        } else {
            format!("{}@{}", username, isp)
        };
        
        // 构造URL参数
        let params = format!(
            "callback=dr1003&DDDDD={}&upass={}&0MKKey=123456",
            full_username,
            encoded_password
        );
        let base_url = &self.config.login_ip;
        let url = format!("{}/drcom/login?{}", base_url.trim_end_matches('/'), params);
        
        let request_builder = self.get_client()
            .get(&url)
            .header("User-Agent", DEFAULT_USER_AGENT)
            .header("Referer", &self.config.login_ip);
            
        let response = request_builder
            .send()
            .await
            .map_err(|e| AppError::NetworkError { source: crate::core::error::map_reqwest_error(e) })?;
            
        let status = response.status();
        
        // 为了安全起见仍检查大小
        let content_length = response.content_length().unwrap_or(0) as usize;
        if content_length > MAX_RESPONSE_SIZE {
            return Err(AppError::NetworkError { source: NetworkError::Other("响应体过大".to_string()) });
        }
        
        let text = response.text().await.map_err(|e| AppError::NetworkError { source: NetworkError::Other(e.to_string()) })?;
        let relevant_text = if text.len() > 8192 {
            text[..8192].to_string()
        } else {
            text
        };
        if is_login_successful(&relevant_text) {
            Ok(relevant_text)
        } else if !status.is_success() {
            Err(AppError::NetworkError { source: NetworkError::HttpError(format!("HTTP错误: 状态码 {}, 响应内容: {}", status, relevant_text)) })
        } else if relevant_text.contains("ldap auth error") || relevant_text.contains("Msg=01") {
            Err(AppError::NetworkError { source: NetworkError::HttpError(format!("认证失败: {}", relevant_text)) })
        } else {
            // 其他情况返回响应内容供上层判断
            Ok(relevant_text)
        }
    }
}

/// 判断登录是否成功
pub fn is_login_successful(login_text: &str) -> bool {
    login_text.contains("注销页") || 
    login_text.contains("认证成功页") || 
    login_text.contains("Dr.COMWebLoginID_3.htm") || 
    login_text.contains("\"result\":1")
}

#[async_trait]
impl NetworkManagerTrait for NetworkManager {
    async fn check_network_status(&self) -> AppResult<NetworkStatus> {
        self.check_network_status().await
    }
    
    async fn check_network_status_with_options(&self, _should_log: bool) -> AppResult<NetworkStatus> {
        self.check_network_status().await
    }
    
    async fn attempt_login_with_credentials(&self, username: &str, password: &str, isp: &str) 
        -> AppResult<String> {
        self.attempt_login_with_credentials(username, password, isp).await
    }
    
    fn clone_box(&self) -> Box<dyn NetworkManagerTrait> {
        Box::new(self.clone())
    }
}