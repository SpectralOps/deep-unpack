//! packing archive files from folders
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use anyhow::{Context, Result};
use derive_builder::Builder;
use ignore::WalkBuilder;
use lazy_static::lazy_static;
use rayon::prelude::*;

use crate::{
    data::{NoWalkList, UnpackStatus},
    formats::kinds::ArchiveKind,
};

/// Skip searching archive file from a list of directories
const NO_WALK_YAML: &str = include_str!("./no_walk.yaml");

lazy_static! {
    pub static ref NO_WALK_LIST: NoWalkList = serde_yaml::from_str(NO_WALK_YAML).unwrap();
}

#[derive(Debug, Clone)]
/// List of archive files that detected during the walk-on directories
struct WalkResult {
    pub archive_kind: ArchiveKind,
    pub path_buf: PathBuf,
}

#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct DeepWalk {
    #[builder(default = "\".\".to_string()")]
    pub folder: String,
    #[builder(default = "\"deep_unpack\".to_string()")]
    pub unpack_folder: String,
    #[builder(default = "self.default_no_walk()")]
    pub no_walk: NoWalkList,
    #[builder(default = "1", field(type = "u32"))]
    pub unpack_level: u32,
    #[builder(default = "\"__${FILENAME}$__\".to_string()")]
    pub extract_template: String,
}

impl DeepWalk {
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> DeepWalkBuilder {
        DeepWalkBuilder::default()
    }
}

impl DeepWalkBuilder {
    #[allow(clippy::unused_self)]
    fn default_no_walk(&self) -> NoWalkList {
        NO_WALK_LIST.clone()
    }

    pub fn extract(&self) -> Result<Vec<UnpackStatus>> {
        let unpack_config = self.build()?;

        let root_path = Path::new(&unpack_config.folder);
        let unpack_folder = Path::new(&unpack_config.unpack_folder);

        // first, find archive files from all the root path directories.
        let walk_result = Self::find_comppress_files(&root_path, unpack_config.no_walk.clone());

        if walk_result.is_empty() {
            return Ok(vec![]);
        }

        let mut result: Vec<UnpackStatus> = vec![];
        result.extend(Self::parallel_unpack(
            walk_result,
            Some(root_path),
            unpack_folder,
            &unpack_config.extract_template,
        ));

        // if a deep level is bigger than 1, search in the extracted folder if there are
        // more archive files. If yes, extract them also
        let mut unpacked_files: Vec<String> = vec![];
        for _ in 2..=unpack_config.unpack_level {
            let walk_result =
                Self::find_comppress_files(&unpack_folder, unpack_config.no_walk.clone())
                    .iter()
                    .filter(|f| {
                        // make sure that we are not unpacking the same file twice
                        let path_str = f.path_buf.display().to_string();
                        if unpacked_files.contains(&path_str) {
                            false
                        } else {
                            unpacked_files.push(path_str);
                            true
                        }
                    })
                    .cloned()
                    .collect::<Vec<_>>();

            if walk_result.is_empty() {
                break;
            }
            result.extend(Self::parallel_unpack(
                walk_result,
                None,
                unpack_folder,
                &unpack_config.extract_template,
            ));
        }
        Ok(result)
    }

    /// Return list of archive files from a given folder
    fn find_comppress_files<P: AsRef<Path>>(path: &P, no_walk: NoWalkList) -> Vec<WalkResult> {
        let (tx, rx) = mpsc::channel();
        WalkBuilder::new(path)
            .filter_entry(move |entry| {
                if let Some(ep) = entry.path().to_str() {
                    if no_walk.ignores.iter().any(|item| item.is_match(ep)) {
                        return false;
                    }
                }
                true
            })
            .hidden(false)
            .git_ignore(true)
            .threads(num_cpus::get())
            .build_parallel()
            .run(move || {
                let tx = tx.clone();
                Box::new(move |result| {
                    if let Ok(de) = result {
                        let metadata = match de.metadata() {
                            Ok(m) => m,
                            Err(e) => {
                                log::info!("could not get dir entry medatada. {}", e);
                                return ignore::WalkState::Continue;
                            }
                        };

                        if metadata.is_dir() {
                            return ignore::WalkState::Continue;
                        }

                        // check if the file is comppreesed file
                        let path_buf = de.path().to_path_buf();
                        if let Some(archive_kind) = ArchiveKind::for_path(&path_buf) {
                            if let Err(err) = tx.send(WalkResult {
                                archive_kind,
                                path_buf,
                            }) {
                                log::info!(
                                    "could not send extract status struct to channel. {}",
                                    err
                                );
                            }
                        }
                    }
                    ignore::WalkState::Continue
                })
            });
        rx.iter().into_iter().collect::<Vec<WalkResult>>()
    }

    /// unpack list of [`WalkResult`] in parallel
    fn parallel_unpack(
        walk_result: Vec<WalkResult>,
        root_path: Option<&Path>,
        unpack_root_folder: &Path,
        extract_template: &str,
    ) -> Vec<UnpackStatus> {
        walk_result
            .par_iter()
            .map(|archive_path| {
                let mut archive = archive_path.archive_kind.new(&archive_path.path_buf);

                let file_unpack_path = match root_path {
                    Some(p) => match archive_path.path_buf.strip_prefix(p) {
                        Ok(a) => unpack_root_folder.join(a),
                        Err(e) => {
                            log::debug!(
                                "could not strip: {} with prefix: {}. err: {}",
                                archive_path.path_buf.display(),
                                p.display(),
                                e
                            );
                            unpack_root_folder.to_path_buf()
                        }
                    },
                    None => archive_path.path_buf.clone(),
                };

                let unpack_folder = match split_file_by_name(file_unpack_path.as_path()) {
                    Ok((file_name, folder)) => {
                        folder.join(extract_template.replace("{FILENAME}", &file_name))
                    }
                    Err(e) => {
                        log::debug!("ould not split file by name. err: {}", e);
                        unpack_root_folder.to_path_buf()
                    }
                };

                match archive.unpack(&unpack_folder) {
                    Ok(()) => UnpackStatus {
                        archive_file: archive_path.path_buf.clone(),
                        extract_to: Some(unpack_folder),
                        err: None,
                    },
                    Err(e) => UnpackStatus {
                        archive_file: archive_path.path_buf.clone(),
                        extract_to: None,
                        err: Some(format!("{}", e)),
                    },
                }
            })
            .collect::<Vec<_>>()
    }
}

/// Split path to file name and parant path
fn split_file_by_name(path: &Path) -> Result<(String, PathBuf)> {
    let file_name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .map(std::string::ToString::to_string)
        .context("could get file name")?;

    let parent = path
        .parent()
        .map(std::path::Path::to_path_buf)
        .context("could get parent file")?;

    Ok((file_name, parent))
}

#[cfg(test)]
mod test_formats_kinds {

    use insta::{assert_debug_snapshot, with_settings};

    use super::*;

    #[test]
    fn can_split_file_by_name() {
        let path = Path::new("foo").join("bar").join("baz.tar.gz");

        with_settings!({filters => vec![
            (r"\\\\", "/"), // for windows
        ]}, {
            assert_debug_snapshot!(split_file_by_name(&path));
        });
    }
}
