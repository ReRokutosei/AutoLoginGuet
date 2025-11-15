#![windows_subsystem = "windows"]
//! AutoLoginGUET 主程序入口

use crate::gui::app::launch_gui;
use autologinguet_core::AppError;
use autologinguet_core::AuthService;
use autologinguet_core::core::config::load_config;
use autologinguet_core::core::error::AppResult;
use std::env;
use std::process;

mod gui;

/// 程序入口函数
fn main() -> AppResult<()> {
    #[cfg(windows)]
    {
        if !gui::webview2_checker::is_webview2_installed() {
            gui::webview2_checker::show_webview2_installation_guide();
            process::exit(1);
        }
    }

    if let Ok(exe_path) = env::current_exe()
        && let Some(exe_dir) = exe_path.parent() {
            let _ = env::set_current_dir(exe_dir);
        }

    let args: Vec<String> = env::args().collect();

    // 只有在通过命令行传递"-silent"参数时才进入静默模式
    let is_silent_mode = args.len() > 1 && args[1] == "-silent";

    if is_silent_mode {
        silent_run()?;
        process::exit(0);
    }

    launch_gui();
    Ok(())
}

/// 静默运行模式的主函数
fn silent_run() -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| AppError::SystemError(format!("创建Tokio Runtime失败: {}", e)))?;

    rt.block_on(async {
        let startup_time = std::time::Instant::now();
        let config = load_config().unwrap_or_default();
        let auth_service = AuthService::new_with_startup_time(config.clone(), Some(startup_time));
        let _ = auth_service.silent_login(config).await?;
        Ok::<(), AppError>(())
    })?;

    Ok(())
}
