//! APIs used by Cargo.
//!
//! These implement the APIs Cargo requires for its commands
//! (such as publish, yank, etc.) to work. The web frontend
//! doesn't use these - it uses the GraphQL interface instead.
pub mod config;
pub mod download;
pub mod index;
pub mod owners;
pub mod publish;
pub mod unyank;
pub mod yank;
