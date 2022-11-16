//! Packing deep archives files from a root folder.
//!
//! ## Usage
//! ```
#![doc = include_str!("../examples/extract-level.rs")]
//!
//! ```
mod data;
mod formats;
mod walk;

pub use data::NoWalkList;
pub use formats::kinds::ArchiveKind;
pub use walk::{extract_to_folder, extract_to_folder_with_ignores};
