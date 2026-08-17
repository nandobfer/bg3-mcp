mod client;
pub mod models;
mod service;

pub(crate) use client::MediaWikiClient;
pub use service::WikiService;
