//! packing archive files from folders
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use anyhow::{Context, Result};
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

pub fn extract_to_folder_with_ignores<P: AsRef<Path>, P1: AsRef<Path>>(
    root_path: &P,
    unpack_root_folder: &P1,
    deep_level: i32,
    no_walk: &NoWalkList,
) -> Vec<UnpackStatus> {
    extract(root_path, unpack_root_folder, deep_level, no_walk)
}

pub fn extract_to_folder<P: AsRef<Path>, P1: AsRef<Path>>(
    root_path: &P,
    unpack_root_folder: &P1,
    deep_level: i32,
) -> Vec<UnpackStatus> {
    extract(root_path, unpack_root_folder, deep_level, &NO_WALK_LIST)
}
/// Extract all archive files in the given path
fn extract<P: AsRef<Path>, P1: AsRef<Path>>(
    root_path: &P,
    unpack_root_folder: &P1,
    deep_level: i32,
    no_walk: &NoWalkList,
) -> Vec<UnpackStatus> {
    let root_path = root_path.as_ref();
    // first, find archive files from all the root path directories.
    let walk_result = find_comppress_files(&root_path, no_walk.clone());

    if walk_result.is_empty() {
        return vec![];
    }

    let mut result: Vec<UnpackStatus> = vec![];
    result.extend(parallel_unpack(
        walk_result,
        Some(root_path),
        unpack_root_folder.as_ref(),
    ));

    // if a deep level is bigger than 1, search in the extracted folder if there are
    // more archive files. If yes, extract them also
    let mut unpacked_files: Vec<String> = vec![];
    for _ in 2..=deep_level {
        let walk_result = find_comppress_files(&unpack_root_folder, no_walk.clone())
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
        result.extend(parallel_unpack(
            walk_result,
            None,
            unpack_root_folder.as_ref(),
        ));
    }
    result
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
                            log::info!("could not send extract status struct to channel. {}", err);
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
) -> Vec<UnpackStatus> {
    walk_result
        .par_iter()
        .map(|archive_path| {
            let mut archive = archive_path.archive_kind.new(&archive_path.path_buf);

            // If root path None, the unpacking destination will be in the same path of the
            // archive file with
            let unpack_folder = root_path.map_or_else(
                || match split_file_by_name(archive_path.path_buf.as_path()) {
                    Ok((file_name, folder)) => folder.join(format!("__{}__", file_name)),
                    Err(e) => {
                        log::debug!("ould not split file by name. err: {}", e);
                        unpack_root_folder.to_path_buf()
                    }
                },
                |r| match &archive_path.path_buf.strip_prefix(r) {
                    Ok(a) => unpack_root_folder.join(a),
                    Err(e) => {
                        log::debug!(
                            "could not strip: {} with prefix: {}. err: {}",
                            archive_path.path_buf.display(),
                            r.display(),
                            e
                        );
                        unpack_root_folder.to_path_buf()
                    }
                },
            );

            match archive.unpack(&unpack_folder) {
                Ok(()) => UnpackStatus {
                    path: archive_path.path_buf.clone(),
                    err: None,
                },
                Err(e) => UnpackStatus {
                    path: archive_path.path_buf.clone(),
                    err: Some(format!("{}", e)),
                },
            }
        })
        .collect::<Vec<_>>()
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
