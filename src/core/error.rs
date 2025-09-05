use thiserror::Error;

/// 应用程序错误类型
#[derive(Debug, Error)]
pub enum AppError {
    /// 网络相关错误，包含所有网络子类型错误
    #[error("网络错误: {source}")]
    NetworkError {
        #[from]
        source: NetworkError,
    },

    /// 配置相关错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 系统相关错误
    #[error("系统错误: {0}")]
    SystemError(String),

    /// 未知错误
    #[error("未知错误: {0}")]
    UnknownError(String),

    /// 密码解密错误（仅用于内部日志，不向用户显示详细信息）
    #[error("{user_msg}")]
    PasswordDecryptionError {
        /// 内部详细错误信息
        internal_msg: String,
        /// 向用户展示友好的错误信息
        user_msg: String,
    },

    /// 通知相关错误
    #[error("通知错误: {0}")]
    NotificationError(String),

    /// 日志相关错误
    #[error("日志错误: {0}")]
    LogError(String),

    /// 加密相关错误
    #[error("加密错误: {0}")]
    CryptoError(String),
}

/// 网络相关错误类型
#[derive(Debug, Error)]
pub enum NetworkError {
    /// DNS解析错误
    #[error("DNS解析失败: {0}")]
    DnsError(String),

    /// 连接超时错误
    #[error("连接超时: {0}")]
    ConnectionTimeout(String),

    /// TLS/SSL错误
    #[error("TLS错误: {0}")]
    TlsError(String),

    /// HTTP错误
    #[error("HTTP错误: {0}")]
    HttpError(String),

    /// 其他网络错误
    #[error("其他网络错误: {0}")]
    Other(String),
}

/// 统一结果类型，用于在模块间传递结果
pub type AppResult<T> = Result<T, AppError>;

/// 将`reqwest`错误转换为`NetworkError`，根据错误特征进行分类
pub fn map_reqwest_error(e: reqwest::Error) -> NetworkError {
    let error_str = e.to_string();
    if error_str.contains("timed out") || error_str.contains("timeout") {
        NetworkError::ConnectionTimeout("请求超时".to_string())
    } else if error_str.contains("dns") || error_str.contains("DNS") {
        NetworkError::DnsError(format!("DNS解析失败: {}", error_str))
    } else if error_str.contains("tls")
        || error_str.contains("TLS")
        || error_str.contains("certificate")
    {
        NetworkError::TlsError(format!("TLS错误: {}", error_str))
    } else if e.is_connect() {
        NetworkError::Other(format!("连接失败: {}", error_str))
    } else {
        NetworkError::HttpError(format!("网络请求失败: {}", error_str))
    }
}

/// 生成用户友好的错误消息
pub fn generate_user_friendly_message(error: &AppError) -> String {
    match error {
        AppError::NetworkError { source } => match source {
            NetworkError::ConnectionTimeout(msg) => {
                format!("连接超时: {}", msg)
            }
            NetworkError::DnsError(msg) => {
                format!("DNS解析失败: {}", msg)
            }
            NetworkError::TlsError(msg) => {
                format!("TLS连接错误: {}", msg)
            }
            NetworkError::HttpError(msg) | NetworkError::Other(msg) => {
                if msg.contains("ldap auth error") || msg.contains("Msg=01") {
                    "登录请求失败: 账号或密码错误".to_string()
                } else {
                    format!("登录请求失败: {}", msg)
                }
            }
        },
        AppError::PasswordDecryptionError { user_msg, .. } => user_msg.clone(),
        _ => format!("登录过程出错: {}", error),
    }
}

/// 生成登录过程的用户友好错误消息
pub fn generate_login_error_message(error: &AppError) -> String {
    match error {
        AppError::NetworkError {
            source: NetworkError::ConnectionTimeout(_),
        } => "连接超时".to_string(),
        AppError::NetworkError {
            source: NetworkError::DnsError(_),
        } => "DNS解析失败".to_string(),
        AppError::NetworkError {
            source: NetworkError::TlsError(_),
        } => "TLS连接错误".to_string(),
        AppError::NetworkError { source } => format!("网络错误: {:?}", source),
        _ => format!("登录过程出错: {}", error),
    }
}

/// 生成网络状态检查的用户友好错误消息
pub fn generate_network_status_error_message(error: &AppError) -> String {
    match error {
        AppError::NetworkError {
            source: NetworkError::ConnectionTimeout(_),
        } => "网络状态检查超时".to_string(),
        AppError::NetworkError {
            source: NetworkError::DnsError(_),
        } => "DNS解析失败，无法解析校园网地址".to_string(),
        AppError::NetworkError {
            source: NetworkError::TlsError(_),
        } => "TLS连接错误".to_string(),
        _ => {
            format!("网络状态检查失败: {}", error)
        }
    }
}
