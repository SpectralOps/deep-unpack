use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::Result;
use zip::read::ZipArchive as ZipArchiveReader;

use crate::formats::kinds::Archive;

#[derive(Debug)]
pub struct ZipArchive {
    path: PathBuf,
}

impl ZipArchive {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }
}

impl Archive for ZipArchive {
    ///  Get the the archive file path
    fn path(&self) -> &Path {
        &self.path
    }

    /// unpack zip file to destination directory
    fn unpack(&mut self, directory: &Path) -> Result<()> {
        let mut rdr = ZipArchiveReader::new(BufReader::new(File::open(&self.path)?))?;
        rdr.extract(directory)?;
        Ok(())
    }
}
