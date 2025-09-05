//! 核心模块

pub mod config;
pub mod crypto;
pub mod dto;
pub mod error;
pub mod events;
pub mod flow;
pub mod message;
pub mod network;
pub mod service;

pub use config::{is_config_complete, load_config, normalize_isp, save_config};
pub use crypto::{
    decrypt_password, decrypt_password_with_machine_key, encrypt_password, generate_machine_key,
};
pub use error::{
    AppError, generate_login_error_message, generate_network_status_error_message,
    generate_user_friendly_message,
};
pub use events::{
    AppEvent, EventHandler, notify_auto_start_set, notify_config_saved, notify_login_attempted,
    notify_network_status_checked,
};
pub use message::MessageCenter;
pub use network::{NetworkManager, is_login_successful};
pub use service::{AuthService, LoginResult};
