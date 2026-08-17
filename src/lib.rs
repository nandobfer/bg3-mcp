pub mod config;
pub mod error;
pub mod infrastructure;
pub mod mcp;
pub mod server;
pub mod wiki;

pub use config::Config;
pub use error::{AppError, WikiError};
pub use server::{build_router, run};
