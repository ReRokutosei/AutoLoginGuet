//! WebView2检查工具

#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::*;

/// 检测是否已安装 WebView2
#[cfg(windows)]
pub fn is_webview2_installed() -> bool {
    const WEBVIEW2_CLIENT_ID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hklm_key_paths = [
        format!(
            "SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{}",
            WEBVIEW2_CLIENT_ID
        ),
        format!(
            "SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{}",
            WEBVIEW2_CLIENT_ID
        ),
    ];

    for hklm_key_path in &hklm_key_paths {
        if check_webview2_key(&hklm, hklm_key_path) {
            return true;
        }
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hkcu_key_path = format!(
        "Software\\Microsoft\\EdgeUpdate\\Clients\\{}",
        WEBVIEW2_CLIENT_ID
    );

    check_webview2_key(&hkcu, &hkcu_key_path)
}

/// 检查指定注册表键中的WebView2安装状态
#[cfg(windows)]
fn check_webview2_key(base_key: &RegKey, key_path: &str) -> bool {
    if let Ok(key) = base_key.open_subkey(key_path) {
        // 如果能打开键，说明 WebView2 相关注册表项存在
        // 尝试获取 pv 值
        if let Ok(version) = key.get_value::<String, _>("pv") {
            // 检查版本号是否有效（不为空且不为 0）
            if !version.is_empty() && version != "0.0.0.0" {
                return true;
            }
        } else {
            // 如果没有 pv 值，但键存在，也可能表示已安装
            // 某些情况下，键存在本身就表明已安装
            return true;
        }
    }

    false
}

/// 显示WebView2安装信息
#[cfg(windows)]
pub fn show_webview2_installation_guide() {
    use win_msgbox::Okay;
    let _ = win_msgbox::show::<Okay>(
        "未检测到 WebView2 Runtime，程序需要此组件才能正常运行。\n\
         请访问微软官网下载并安装 WebView2 Runtime：\n\
         https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/\
         ",
    );
}
