use std::path::PathBuf;

use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug, Clone)]
pub struct NoWalkList {
    #[serde(with = "serde_regex", default)]
    pub ignores: Vec<Regex>,
}

#[derive(Debug, Clone)]
pub struct UnpackStatus {
    pub archive_file: PathBuf,
    pub extract_to: Option<PathBuf>,
    pub err: Option<String>,
}
