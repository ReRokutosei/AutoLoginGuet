use reqwest::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

/// 网络状态枚举，用于更准确地表示当前网络连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkStatus {
    /// 已登录校园网且能访问广域网
    LoggedInAndConnected,
    /// 未登录校园网但能访问广域网
    NotLoggedInButConnected,
    /// 能访问广域网但无法确定校园网状态
    ConnectedToWan,
    /// 网络检查失败
    NetworkCheckFailed,
    /// 无法访问校园网且无法访问广域网
    NoNetwork,
}

/// 网络配置结构体
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct NetworkConfig {
    pub login_ip: String,
    pub result_return: String,
    pub signed_in_title: String,
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
pub struct NetworkManager {
    client: Client,
    config: NetworkConfig,
}

impl NetworkManager {
    pub fn new(config: NetworkConfig) -> Self {
        NetworkManager {
            client: Client::new(),
            config,
        }
    }

    /// 检查校园网状态
    pub async fn check_campus_network_status(&self) -> Result<NetworkStatus, String> {
        let response = self.client
            .get(&self.config.login_ip)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| e.to_string())?;
            
        let status = response.status();
        let text = response.text().await.map_err(|e| e.to_string())?;
        
        if status.is_redirection() {
            Ok(NetworkStatus::NotLoggedInButConnected) // 重定向通常表示未登录
        } else if text.contains(&self.config.signed_in_title) {
            Ok(NetworkStatus::LoggedInAndConnected) // 已登录
        } else if text.contains(&self.config.not_sign_in_title) {
            Ok(NetworkStatus::NotLoggedInButConnected) // 未登录但可以访问校园网登录页面
        } else {
            // 其他情况，可能是已经连接到外网
            Ok(NetworkStatus::ConnectedToWan)
        }
    }

    /// 检查外部网络连接状态
    async fn check_external_connectivity(&self) -> bool {
        let external_sites = [
            "https://www.baidu.com",
            "https://www.qq.com",
            "https://www.sina.com.cn",
            "https://www.alibaba.com",
            "https://www.taobao.com",
        ];
        
        let mut tasks = Vec::new();
        for url in &external_sites {
            let client = self.client.clone();
            let url = url.to_string();
            tasks.push(tokio::spawn(async move {
                let result = client
                    .get(&url)
                    .timeout(Duration::from_secs(5))
                    .send()
                    .await;
                
                match result {
                    Ok(response) => response.status().is_success(),
                    Err(_) => false,
                }
            }));
        }
        
        let results = futures::future::join_all(tasks).await;
        results.into_iter().any(|result| result.unwrap_or(false))
    }


    pub async fn check_network_status(&self) -> Result<NetworkStatus, String> {
        let campus_status = self.check_campus_network_status().await;
        let external_connected = self.check_external_connectivity().await;
        
        match (campus_status, external_connected) {
            (Ok(NetworkStatus::LoggedInAndConnected), true) => {
                Ok(NetworkStatus::LoggedInAndConnected)
            }
            (Ok(NetworkStatus::NotLoggedInButConnected), true) => {
                Ok(NetworkStatus::NotLoggedInButConnected)
            }
            (Ok(NetworkStatus::ConnectedToWan), true) => {
                Ok(NetworkStatus::ConnectedToWan)
            }
            (Err(_), true) => {
                Ok(NetworkStatus::ConnectedToWan)
            }
            (_, false) => {
                // 无法访问外部网络，检查是否能访问校园网
                match self.check_campus_network_status().await {
                    Ok(status) => Ok(status),
                    Err(_) => Ok(NetworkStatus::NetworkCheckFailed),
                }
            }
            _ => Ok(NetworkStatus::NetworkCheckFailed)
        }
    }

    pub async fn attempt_login_with_credentials(&self, username: &str, password: &str, isp: &str) -> Result<String, String> {
        let actual_isp = if isp == "校园网" {
            ""  // 选择"校园网"时，发送空字符串
        } else {
            isp // 其他情况直接使用ISP值
        };
        
        self.try_drcom_login(username, password, actual_isp).await
    }
    
    /// drcom 登录方式
    async fn try_drcom_login(&self, username: &str, password: &str, isp: &str) -> Result<String, String> {
        // URL编码密码
        let encoded_password = encode(password);
        
        // 构造（学号+运营商）
        // - 校园网: 学号 (不带任何后缀)
        // - 中国移动: 学号@cmcc
        // - 中国联通: 学号@unicom
        // - 中国电信: 学号@telecom
        let full_username = format!("{}{}", username, isp);
        
        // 构造URL参数
        let params = format!(
            "callback=dr1003&DDDDD={}&upass={}&0MKKey=123456",
            full_username,
            encoded_password
        );
        let base_url = &self.config.login_ip;
        let url = format!("{}/drcom/login?{}", base_url.trim_end_matches('/'), params);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;
        
        let request_builder = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .header("Referer", &self.config.login_ip);
            
        let response = request_builder
            .send()
            .await
            .map_err(|e| e.to_string())?;
            
        let status = response.status();
        let text = response.text().await.map_err(|e| e.to_string())?;


        if status.is_success() && (text.contains("\"result\":1")) {
            Ok(text)
        } else if !status.is_success() {
            Err(format!("HTTP错误: 状态码 {}, 响应内容: {}", status, text))
        } else if text.contains("ldap auth error") || text.contains("Msg=01") {
            Err(format!("认证失败: {}", text))
        } else {
            // 其他情况返回响应内容供上层判断
            Ok(text)
        }
    }
}