//! Supported archive formats
use std::path::Path;

use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

use crate::formats::zip::ZipArchive;

lazy_static! {
    static ref BY_PATTERN: Vec<(Regex, ArchiveKind)> = vec![
        (Regex::new(r"(?i)\.zip$").unwrap(), ArchiveKind::Zip),
        (Regex::new(r"(?i)\.jar$").unwrap(), ArchiveKind::Zip)
    ];
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArchiveKind {
    Zip,
}

pub trait Archive {
    fn path(&self) -> &Path;

    fn unpack(&mut self, path: &Path) -> Result<()>;
}

impl ArchiveKind {
    /// Check if the given file path is a supported archive type
    #[must_use]
    pub fn for_path(path: &Path) -> Option<Self> {
        Self::determine_by_filename(path)
    }

    /// determine by file name if the path is an archive file
    fn determine_by_filename(path: &Path) -> Option<Self> {
        if let Some(filename) = path.file_name().and_then(std::ffi::OsStr::to_str) {
            for &(ref regex, ty) in BY_PATTERN.iter() {
                if regex.is_match(filename) {
                    return Some(ty);
                }
            }
        };
        None
    }

    // create a new archive trait
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(self, path: &Path) -> Box<dyn Archive> {
        match self {
            Self::Zip => Box::new(ZipArchive::new(path)),
        }
    }
}
