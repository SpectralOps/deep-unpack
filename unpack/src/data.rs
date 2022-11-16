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
    pub path: PathBuf,
    pub err: Option<String>,
}
