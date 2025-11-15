//! 流量信息查询模块
//!
//! 该模块负责登录到用户自助服务系统并获取剩余流量信息
//! 该模块访问的系统与登录校园网的系统不同，各自独立

use regex::Regex;
use reqwest::Client;
use reqwest::cookie::Jar;
use std::sync::Arc;
use std::time::Duration;

/// 流量服务错误类型
#[derive(Debug)]
pub enum FlowError {
    NetworkError(String),
    ParseError(String),
    LoginFailed(String),
}

impl std::fmt::Display for FlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlowError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            FlowError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            FlowError::LoginFailed(msg) => write!(f, "登录失败: {}", msg),
        }
    }
}

impl std::error::Error for FlowError {}

/// 用户流量信息
#[derive(Debug, Clone)]
pub struct UserFlowInfo {
    /// 剩余流量 (MB)
    pub left_flow: f64,
}

impl UserFlowInfo {
    pub fn new(left_flow: f64) -> Self {
        Self { left_flow }
    }

    /// 将剩余流量转换为GB并保留两位小数
    pub fn left_flow_gb(&self) -> f64 {
        (self.left_flow / 1024.0 * 100.0).round() / 100.0
    }

    /// 格式化剩余流量信息，根据大小选择合适的单位
    /// 根据流量大小自动选择合适的单位：
    /// - 流量为0时显示"流量耗尽，限速不限量生效"
    /// - 1MB到1024MB之间显示MB
    /// - 大于1024MB显示GB
    pub fn format_flow_info(&self) -> String {
        if self.left_flow == 0.0 {
            "流量耗尽，限速不限量生效".to_string()
        } else if self.left_flow < 1024.0 {
            // 1MB到1024MB之间，直接显示MB
            format!("剩余流量 {:.2} MB", self.left_flow)
        } else {
            // 大于1024MB，转换为GB显示
            format!("剩余流量 {:.2} GB", self.left_flow_gb())
        }
    }
}

/// 流量服务
#[derive(Clone)]
pub struct FlowService {
    client: Client,
    base_url: String,
}

impl FlowService {
    /// 创建新的流量服务实例
    pub fn new() -> Self {
        Self::with_client(None)
    }

    /// 使用指定的HTTP客户端创建流量服务实例
    /// 如果client为None，则创建一个新的客户端
    /// 必须带有cookie，否则会失败
    pub fn with_client(client: Option<Client>) -> Self {
        let client = client.unwrap_or_else(|| {
            let cookie_store = Arc::new(Jar::default());
            Client::builder()
                .timeout(Duration::from_secs(10))
                .cookie_provider(Arc::clone(&cookie_store))
                .pool_max_idle_per_host(0)
                .tcp_keepalive(None)
                .build()
                .expect("Failed to create HTTP client")
        });

        let base_url = "https://nicdrcom.guet.edu.cn/Self".to_string();

        Self { client, base_url }
    }

    /// 登录用户自助服务系统并获取流量信息
    ///
    /// 该函数通过模拟用户登录流程来获取流量信息，具体步骤如下：
    /// 1. 获取登录页面以提取checkcode（验证码标识）
    /// 2. 访问验证码图片URL（不需要保存图片，只需访问）
    ///     > 非常奇怪的机制，图片验证码只在第三级的流量充值页面出现
    ///     >
    ///     > 但是在登录页面不先访问该图片验证码并传递四位纯数字的话
    ///     >
    ///     > 就无法登录进去。这四位数字也不必和图片对应......
    /// 3. 生成随机四位数字验证码
    /// 4. 对密码进行MD5加密
    /// 5. 构造登录数据并执行登录
    /// 6. 从仪表板页面提取剩余流量信息
    ///
    /// # 参数
    /// * `account` - 用户账号
    /// * `password` - 用户密码（明文）
    ///
    /// # 返回值
    /// 成功时返回包含用户流量信息的UserFlowInfo结构体，失败时返回FlowError错误
    pub async fn get_user_flow_info(
        &self,
        account: &str,
        password: &str,
    ) -> Result<UserFlowInfo, FlowError> {
        // 1. 获取登录页面以提取checkcode
        let login_page = self.get_login_page().await?;

        let checkcode = self
            .extract_checkcode_from_login_page(&login_page)
            .ok_or_else(|| FlowError::ParseError("无法提取checkcode".to_string()))?;

        // 2. 访问验证码图片URL（不需要保存图片，只需访问）
        self.access_captcha_image().await?;

        // 3. 生成随机四位数字验证码
        let random_code = self.generate_random_code();

        // 4. 对密码进行MD5加密
        let encrypted_password = self.md5_encrypt(password);

        // 5. 构造登录数据并执行登录
        let login_data = [
            ("account", account),
            ("password", &encrypted_password),
            ("checkcode", &checkcode),
            ("code", &random_code),
        ];

        let dashboard_content = self.login(&login_data).await?;

        // 6. 从仪表板页面提取剩余流量信息
        let left_flow = self
            .extract_remaining_flow_from_user_info(&dashboard_content)
            .ok_or_else(|| FlowError::ParseError("无法提取剩余流量信息".to_string()))?;

        Ok(UserFlowInfo::new(left_flow))
    }

    /// 创建带有通用请求头的GET请求构建器
    fn create_request_builder(&self, url: &str) -> reqwest::RequestBuilder {
        self.client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
            .header("Accept-Language", "zh-CN,zh;q=0.9")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
    }

    /// 获取登录页面内容
    async fn get_login_page(&self) -> Result<String, FlowError> {
        let url = format!("{}/login/", self.base_url);
        self.create_request_builder(&url)
            .send()
            .await
            .map_err(|e| FlowError::NetworkError(format!("获取登录页面失败: {}", e)))?
            .text()
            .await
            .map_err(|e| FlowError::NetworkError(format!("读取登录页面内容失败: {}", e)))
    }

    /// 从登录页面提取隐藏的checkcode值
    fn extract_checkcode_from_login_page(&self, login_page_content: &str) -> Option<String> {
        let re = Regex::new(r#"name="checkcode" value="([^"]*)""#).ok()?;
        if let Some(captures) = re.captures(login_page_content) {
            Some(captures.get(1)?.as_str().to_string())
        } else {
            None
        }
    }

    /// 访问验证码图片URL
    async fn access_captcha_image(&self) -> Result<(), FlowError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let url = format!("{}/login/randomCode?t={}", self.base_url, timestamp);

        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| FlowError::NetworkError(format!("访问验证码图片失败: {}", e)))?;

        Ok(())
    }

    /// 生成随机四位数字验证码
    fn generate_random_code(&self) -> String {
        use rand::Rng;
        let mut rng = rand::rng();
        format!("{:04}", rng.random_range(0..=9999))
    }

    /// MD5加密
    fn md5_encrypt(&self, text: &str) -> String {
        let digest = md5::compute(text.as_bytes());
        format!("{:x}", digest)
    }

    /// 执行登录操作
    async fn login(&self, login_data: &[(&str, &str)]) -> Result<String, FlowError> {
        let login_url = format!("{}/login/verify", self.base_url);

        let login_response = self.client
            .post(&login_url)
            .form(login_data)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
            .header("Accept-Language", "zh-CN,zh;q=0.9")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .map_err(|e| FlowError::NetworkError(format!("发送登录请求失败: {}", e)))?;

        let _login_response_text = login_response
            .text()
            .await
            .map_err(|e| FlowError::NetworkError(format!("读取登录响应内容失败: {}", e)))?;

        // 获取仪表板页面内容以验证登录是否成功
        let dashboard_url = format!("{}/dashboard", self.base_url);

        let dashboard_response = self.client
            .get(&dashboard_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
            .header("Accept-Language", "zh-CN,zh;q=0.9")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .map_err(|e| FlowError::NetworkError(format!("获取仪表板页面失败: {}", e)))?;

        let dashboard_content = dashboard_response
            .text()
            .await
            .map_err(|e| FlowError::NetworkError(format!("读取仪表板页面内容失败: {}", e)))?;

        // 检查仪表板页面是否包含登录成功标志
        if dashboard_content.contains("leftFlow") {
            Ok(dashboard_content)
        } else if dashboard_content.contains("用户自助服务系统")
            && dashboard_content.contains("account")
        {
            // 检查是否仍然在登录页面（说明登录失败）
            Err(FlowError::LoginFailed(
                "登录失败，账号或密码错误".to_string(),
            ))
        } else {
            // 其他情况
            Err(FlowError::LoginFailed(
                "登录失败，未在仪表板页面找到流量信息".to_string(),
            ))
        }
    }

    /// 从用户自助服务系统的json数据中提取剩余流量信息
    ///
    /// # 参数
    /// * `user_info_text` - 包含用户信息的JavaScript代码文本
    /// * `pattern` - 用于提取流量信息的正则表达式
    ///
    /// # 返回值
    /// 返回剩余流量的MB数，如果无法解析则返回None
    fn extract_flow_with_regex(&self, user_info_text: &str, pattern: &str) -> Option<f64> {
        let re = Regex::new(pattern).ok()?;
        if let Some(captures) = re.captures(user_info_text) {
            if let Some(flow_str) = captures.get(1) {
                flow_str.as_str().parse::<f64>().ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 从用户自助服务系统的json数据中提取剩余流量信息
    ///
    /// # 参数
    /// * `user_info_text` - 包含用户信息的JavaScript代码文本
    ///
    /// # 返回值
    /// 返回剩余流量的MB数，如果无法解析则返回None
    fn extract_remaining_flow_from_user_info(&self, user_info_text: &str) -> Option<f64> {
        // 优先使用HTML div标签方式提取流量信息
        // js方式获取的数据与div方式存在差异，可能是服务器未及时更新js中的信息导致
        // 为了获取到准确的剩余流量值，优先使用div标签
        let div_pattern =
            r#"<dt>\s*(\d+(?:\.\d+)?)\s*<small[^>]*>M</small>\s*</dt>\s*<dd>剩余流量</dd>"#;
        let div_result = self.extract_flow_with_regex(user_info_text, div_pattern);

        // 如果div方式失败，则使用js方式提取
        if div_result.is_none() {
            let js_pattern = r#""leftFlow":(\d+\.?\d*)"#;
            // 返回js方式的结果
            return self.extract_flow_with_regex(user_info_text, js_pattern);
        }

        // 返回div方式的结果
        div_result
    }
}

impl Default for FlowService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_user_flow_info() {
        // 测试时需要替换为实际的账号和密码
        // 后续如果校方把接口变动，请自行修改。但估计短时间内是不会改的
        // "userGroupId":13,"userGroupName":"学生组-大一"
        // "userGroupId":14,"userGroupName":"学生组-大二"
        // "userGroupId":15,"userGroupName":"学生组-大三"
        // "userGroupId":16,"userGroupName":"学生组-大四"
        let account = "xxx";
        let password = "xxx";

        let flow_service = FlowService::new();
        match flow_service.get_user_flow_info(account, password).await {
            Ok(flow_info) => {
                println!("剩余流量：{:?} MB", flow_info)
            }
            Err(e) => {
                println!("获取流量信息失败：{}", e);
            }
        }
    }
}
