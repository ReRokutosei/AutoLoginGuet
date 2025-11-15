//! 网络管理模块
//!
//! 负责处理应用程序的所有网络相关功能

use crate::core::config::normalize_isp;
use crate::core::error::{AppError, AppResult, NetworkError};
use crate::core::message::{CampusNetworkStatus, WanStatus};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use urlencoding::encode;

const EXTERNAL_SITES: [&str; 5] = [
    "https://www.baidu.com",
    "https://www.qq.com",
    "https://www.sina.com.cn",
    "https://www.alibaba.com",
    "https://www.bytedance.com/",
];

// 限制响应体大小为10MB（对于校园网认证足够大，同时防止极端情况）
const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36";

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
    pub async fn check_campus_network(&self) -> AppResult<CampusNetworkStatus> {
        let response = self
            .get_client()
            .get(&self.config.login_ip)
            .send()
            .await
            .map_err(|e| AppError::NetworkError {
                source: crate::core::error::map_reqwest_error(e),
            })?;

        let text = response.text().await.map_err(|e| AppError::NetworkError {
            source: NetworkError::Other(e.to_string()),
        })?;

        let truncated_text = if text.len() > 4096 {
            text[..4096].to_string()
        } else {
            text
        };

        if truncated_text.contains(&self.config.signed_in_title) {
            Ok(CampusNetworkStatus::AlreadyLoggedIn)
        } else if truncated_text.contains(&self.config.not_sign_in_title) {
            Ok(CampusNetworkStatus::NotLoggedIn)
        } else {
            // 当无法识别校园网页面时，视为未登录
            Ok(CampusNetworkStatus::NotLoggedIn)
        }
    }

    /// 检查广域网状态
    pub async fn check_wan_network(&self) -> WanStatus {
        let result = self.check_external_connectivity().await;

        if result {
            WanStatus::Connected
        } else {
            WanStatus::Disconnected
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

                let result =
                    tokio::time::timeout(Duration::from_secs(5), client.get(site).send()).await;

                match result {
                    Ok(Ok(response)) => {
                        let status = response.status();

                        // 检查是否被重定向到校园网登录页面
                        if let Some(url) = response.url().host_str()
                            && login_ip.contains(url)
                        {
                            // 被重定向到校园网登录页面，说明未连接外网
                            return false;
                        }

                        // 检查响应内容是否包含登录页面标题
                        let text_result =
                            tokio::time::timeout(Duration::from_secs(3), response.text()).await;

                        if let Ok(Ok(text)) = text_result
                            && text.contains(&not_sign_in_title)
                        {
                            // 响应内容是登录页面，说明未连接外网
                            return false;
                        }

                        status.is_success()
                    }
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

    /// 使用凭据尝试登录
    pub async fn attempt_login_with_credentials(
        &self,
        username: &str,
        password: &str,
        isp: &str,
    ) -> AppResult<LoginResult> {
        let start_time = std::time::Instant::now();

        let result = self
            .try_drcom_login(username, password, &normalize_isp(isp))
            .await;

        let elapsed_time = start_time.elapsed().as_secs_f64();

        match result {
            Ok(response_text) => {
                if is_login_successful(&response_text) {
                    Ok(LoginResult {
                        success: true,
                        campus_status: CampusNetworkStatus::LoginSuccess,
                        elapsed_time,
                    })
                } else {
                    Ok(LoginResult {
                        success: false,
                        campus_status: CampusNetworkStatus::NotLoggedIn,
                        elapsed_time,
                    })
                }
            }
            Err(_e) => Ok(LoginResult {
                success: false,
                campus_status: CampusNetworkStatus::NotLoggedIn,
                elapsed_time,
            }),
        }
    }

    async fn try_drcom_login(
        &self,
        username: &str,
        password: &str,
        isp: &str,
    ) -> AppResult<String> {
        // URL编码密码
        let encoded_password = encode(password);

        // 构造（学/工号+运营商）
        let full_username = if isp.is_empty() {
            username.to_string()
        } else {
            format!("{}@{}", username, isp)
        };

        // 构造URL参数
        let params = format!(
            "callback=dr1003&DDDDD={}&upass={}&0MKKey=123456",
            full_username, encoded_password
        );
        let base_url = &self.config.login_ip;
        let url = format!("{}/drcom/login?{}", base_url.trim_end_matches('/'), params);

        let request_builder = self
            .get_client()
            .get(&url)
            .header("User-Agent", DEFAULT_USER_AGENT)
            .header("Referer", &self.config.login_ip);

        let response = request_builder
            .send()
            .await
            .map_err(|e| AppError::NetworkError {
                source: crate::core::error::map_reqwest_error(e),
            })?;

        let content_length = response.content_length().unwrap_or(0) as usize;
        if content_length > MAX_RESPONSE_SIZE {
            return Err(AppError::NetworkError {
                source: NetworkError::Other("响应体过大".to_string()),
            });
        }

        let text = response.text().await.map_err(|e| AppError::NetworkError {
            source: NetworkError::Other(e.to_string()),
        })?;

        let relevant_text = if text.len() > 8192 {
            text[..8192].to_string()
        } else {
            text
        };

        Ok(relevant_text)
    }
}

/// 登录结果
#[derive(Debug, Clone)]
pub struct LoginResult {
    /// 是否登录成功
    pub success: bool,
    /// 校园网状态
    pub campus_status: CampusNetworkStatus,
    /// 耗时（秒）
    pub elapsed_time: f64,
}

/// 判断登录是否成功
pub fn is_login_successful(login_text: &str) -> bool {
    login_text.contains("注销页")
        || login_text.contains("认证成功页")
        || login_text.contains("Dr.COMWebLoginID_3.htm")
        || login_text.contains("\"result\":1")
}

#[async_trait]
pub trait NetworkManagerTrait: Send + Sync {
    /// 检查校园网状态
    async fn check_campus_network(&self) -> AppResult<CampusNetworkStatus>;

    /// 检查广域网状态
    async fn check_wan_network(&self) -> WanStatus;

    /// 使用凭据尝试登录
    async fn attempt_login_with_credentials(
        &self,
        username: &str,
        password: &str,
        isp: &str,
    ) -> AppResult<LoginResult>;

    /// 克隆网络管理器
    fn clone_box(&self) -> Box<dyn NetworkManagerTrait>;
}

#[async_trait]
impl NetworkManagerTrait for NetworkManager {
    async fn check_campus_network(&self) -> AppResult<CampusNetworkStatus> {
        self.check_campus_network().await
    }

    async fn check_wan_network(&self) -> WanStatus {
        self.check_wan_network().await
    }

    async fn attempt_login_with_credentials(
        &self,
        username: &str,
        password: &str,
        isp: &str,
    ) -> AppResult<LoginResult> {
        self.attempt_login_with_credentials(username, password, isp)
            .await
    }

    fn clone_box(&self) -> Box<dyn NetworkManagerTrait> {
        Box::new(self.clone())
    }
}
