//! 核心模块

pub mod config;
pub mod crypto;
pub mod dto;
pub mod error;
pub mod events;
pub mod network;
pub mod service;
pub mod message;

pub use config::{load_config, save_config, is_config_complete, normalize_isp};
pub use network::{NetworkManager, is_login_successful};
pub use crypto::{encrypt_password, decrypt_password, generate_machine_key, decrypt_password_with_machine_key};
pub use service::{AuthService, LoginResult};
pub use error::{AppError, generate_user_friendly_message, generate_network_status_error_message, generate_login_error_message};
pub use events::{AppEvent, EventHandler, DefaultEventHandler, notify_network_status_checked, notify_login_attempted, notify_config_saved, notify_auto_start_set};
pub use message::MessageCenter;