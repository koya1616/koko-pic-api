//! Email sending functionality module
//!
//! This module provides basic email sending capabilities using lettre,
//! a popular email library for Rust.

mod service;
mod types;

pub use service::EmailService;
pub use types::{EmailMessage, SmtpConfig};
