//! AutoLoginGUET 核心库

pub mod core;

pub use core::config::normalize_isp;
pub use core::dto::GuiConfigDto;
pub use core::error::AppError;
pub use core::events::{AppEvent, EventHandler};
pub use core::message::MessageCenter;
pub use core::service::{AuthService, LoginResult};

// GUI相关导出
#[cfg(feature = "gui")]
pub mod gui;

#[cfg(feature = "gui")]
pub use core::events::GuiEventHandlerMessage;
