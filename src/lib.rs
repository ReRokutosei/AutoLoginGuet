//! AutoLoginGUET 核心库

pub mod core;

pub use core::dto::GuiConfigDto;
pub use core::service::{AuthService, LoginResult};
pub use core::error::AppError;
pub use core::events::{AppEvent, EventHandler, DefaultEventHandler};
pub use core::config::normalize_isp;
pub use core::message::MessageCenter;

// GUI相关导出
#[cfg(feature = "gui")]
pub mod gui;

#[cfg(feature = "gui")]
pub use core::events::GuiEventHandlerMessage;