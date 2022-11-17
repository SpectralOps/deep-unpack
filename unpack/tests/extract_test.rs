use std::{
    env::temp_dir,
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
};

use deep_unpack::{ArchiveKind, DeepWalk, NoWalkList};
use ignore::WalkBuilder;
use insta::{assert_debug_snapshot, with_settings};
use regex::Regex;
use rstest::rstest;
use uuid::Uuid;

fn get_temp_dir() -> PathBuf {
    let dir = temp_dir();
    let temp_folder = dir.join(Uuid::new_v4().to_string());
    fs::create_dir(&temp_folder).unwrap();
    temp_folder
}

fn get_files_from_folder<P: AsRef<Path>>(path: &P) -> Vec<String> {
    let (tx, rx) = mpsc::channel();
    WalkBuilder::new(path).build_parallel().run(move || {
        let tx = tx.clone();
        Box::new(move |result| {
            if let Ok(de) = result {
                let metadata = de.metadata().unwrap();
                if metadata.is_file() {
                    tx.send(de.path().display().to_string()).unwrap();
                }
            }
            ignore::WalkState::Continue
        })
    });
    {
        let mut result = rx.iter().into_iter().collect::<Vec<String>>();
        result.sort();
        result
    }
}

macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
}

#[rstest]
#[case(1)]
#[case(2)]
#[case(3)]
fn test_can_extract_with_unpack_level(#[case] unpack_level: u32) {
    set_snapshot_suffix!("[level]-[{}]", unpack_level);

    let destination_folder = get_temp_dir();
    let path = Path::new("tests").join("mocks").join("multiple");

    let results = {
        let mut r = DeepWalk::new()
            .folder(format!("{}", path.display()))
            .unpack_folder(format!("{}", destination_folder.join("dest").display()))
            .unpack_level(unpack_level)
            .extract()
            .unwrap();

        r.sort_by(|a, b| a.archive_file.cmp(&b.archive_file));
        r
    };

    with_settings!({filters => vec![
        (r"//*.+/(dest)", "[DYNAMIC-PATH]"),
        (r"([C]?:\\.+dest\\\\)", "[DYNAMIC-PATH]/"),// for windows
        (r"\\\\", "/"), // for windows
    ]}, {
        assert_debug_snapshot!(get_files_from_folder(&destination_folder));
        assert_debug_snapshot!(results);
    });
    fs::remove_dir_all(destination_folder).unwrap();
}

#[test]
fn test_can_extract_with_ignores() {
    let destination_folder = get_temp_dir();
    let path = Path::new("tests").join("mocks").join("multiple");

    let no_walk = NoWalkList {
        ignores: vec![
            Regex::new("folder-1").unwrap(),
            Regex::new("folder-2").unwrap(),
        ],
    };
    let results = {
        let mut r = DeepWalk::new()
            .folder(format!("{}", path.display()))
            .unpack_folder(format!("{}", destination_folder.join("dest").display()))
            .no_walk(no_walk)
            .extract()
            .unwrap();

        r.sort_by(|a, b| a.archive_file.cmp(&b.archive_file));
        r
    };

    with_settings!({filters => vec![
            (r"//*.+/(dest)", "[DYNAMIC-PATH]"),
            (r"([C]?:\\.+dest\\\\)", "[DYNAMIC-PATH]/"),// for windows
            (r"\\\\", "/"), // for windows
        ]}, {
            assert_debug_snapshot!(get_files_from_folder(&
    destination_folder));         assert_debug_snapshot!(results);
        });
    fs::remove_dir_all(destination_folder).unwrap();
}

#[test]
fn test_can_extract_with_template() {
    let destination_folder = get_temp_dir();
    let path = Path::new("tests").join("mocks").join("multiple");

    let results = {
        let mut r = DeepWalk::new()
            .folder(format!("{}", path.display()))
            .unpack_folder(format!("{}", destination_folder.join("dest").display()))
            .unpack_level(2 as u32)
            .extract_template("_PREFIX_${FILENAME}$_SUFFIX_")
            .extract()
            .unwrap();

        r.sort_by(|a, b| a.archive_file.cmp(&b.archive_file));
        r
    };

    with_settings!({filters => vec![
            (r"//*.+/(dest)", "[DYNAMIC-PATH]"),
            (r"([C]?:\\.+dest\\\\)", "[DYNAMIC-PATH]/"),// for windows
            (r"\\\\", "/"), // for windows
        ]}, {
            assert_debug_snapshot!(get_files_from_folder(&
    destination_folder));         assert_debug_snapshot!(results);
        });
    fs::remove_dir_all(destination_folder).unwrap();
}

#[rstest]
#[case("zip", "archive.zip")]
#[case("jar", "archive.jar")]
fn test_can_extract_format(#[case] folder: &str, #[case] file: &str) {
    set_snapshot_suffix!("[{}]-[{}]", folder, file);

    let destination_folder = get_temp_dir();
    let file_path = Path::new("tests").join("mocks").join(folder).join(file);

    let archive_kind = ArchiveKind::for_path(&file_path);
    assert_debug_snapshot!(archive_kind);

    archive_kind
        .unwrap()
        .new(&file_path)
        .unpack(&destination_folder.join("dest"))
        .unwrap();

    with_settings!({filters => vec![
        (r"//*.+/(dest)", "[DYNAMIC-PATH]"),
        (r"([C]?:\\.+dest\\\\)", "[DYNAMIC-PATH]/"),// for windows
        (r"\\\\", "/"), // for windows
    ]}, {
        assert_debug_snapshot!(get_files_from_folder(&destination_folder));
    });
    fs::remove_dir_all(destination_folder).unwrap();
}
