//! Packing deep archives files from a root folder.
//!
//! ## Usage
//! ```
#![doc = include_str!("../examples/extract-level.rs")]
//!
//! ```
mod data;
mod formats;
mod unpack;

pub use data::{NoWalkList, UnpackStatus};
pub use formats::kinds::ArchiveKind;
pub use unpack::DeepWalk;
