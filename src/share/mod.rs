//! Local document-sharing system.
//!
//! Generates self-contained HTML documentation for a project / folder / single
//! request and hosts it on a localhost HTTP server with **strict** access
//! control (expiration + password). The same server also serves Mock responses
//! for configured rules.
//!
//! Modules:
//! - [`models`] — `ShareConfig` and friends.
//! - [`persist`] — `shares.json` load/save.
//! - [`html`] — postman-style HTML document generation.
//! - [`server`] — the HTTP server core (routing + strict enforcement).
//! - [`qrcode`] — QR code generation for the 二维码 share method.

pub mod html;
pub mod models;
pub mod persist;
pub mod qrcode;
pub mod server;

pub use models::{AccessControl, Expiration, FieldDisplay, ShareConfig, ShareMethod, ShareScope};
