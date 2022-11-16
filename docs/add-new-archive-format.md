## Add new archive format

Follow those steps for supporting a new archive format

1. Add archive format file under [formats](./unpack/src/formats) folder with the name formats/{archive-kind}.rs. You can use this boilerplate:
```rs
#[derive(Debug)]
pub struct [ArchiveFormat]Archive {
    path: PathBuf,
}

impl [ArchiveFormat]Archive {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }
}

impl Archive for [ArchiveFormat]Archive {
    ///  Get the the archive file path
    fn path(&self) -> &Path {
        &self.path
    }

    /// unpack archive file to destination directory
    fn unpack(&mut self, directory: &Path) -> Result<()> {
        // TODO
    }
}
```
2. Go to [kinds.rs](./unpack/src/formats/kinds.rs):
    1. Extension archive file to [`BY_PATTERN`] cosnt variable.
    2. Add archive format to [`ArchiveKind`] enum
    3. In the `open` function, add the new [`ArchiveKind`] format to initialize the new archive format


3. Adding testing:
    1. Add a new archive file format to [mocks](../unpack/tests/mocks) 
    2. got to [extract_test.rs](../unpack/tests/extract_test.rs) to `text_can_extract_format` and add a new case test at the top of the function:
        ```rs
        #[rstest]
        #[case("zip", "archive.zip")]
        #[case("jar", "archive.jar")]
        #[case("[new format type]", "archive.[archive extension]")]
        fn text_can_extract_format(#[case] folder: &str, #[case] file: &str) {
            ....
        ```

    3. [Run test](../CONTRIBUTING.md#unitest)

4. Add example file to [examples](../unpack/examples/) folder