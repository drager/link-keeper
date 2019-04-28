use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct FileHandling {
    path: PathBuf,
    file_name: String,
}

impl FileHandling {
    pub fn new(path: PathBuf, file_name: String) -> Self {
        Self { path, file_name }
    }

    pub fn file_exists(&self) -> bool {
        let full_path = self.joined();

        full_path.exists()
    }

    pub fn create_file(&self) -> Result<(), io::Error> {
        if !self.file_exists() {
            fs::File::create(self.joined())?;
        }

        Ok(())
    }

    pub fn joined(&self) -> PathBuf {
        self.path.join(&self.file_name)
    }

    pub fn read_data_from_file(&self) -> Result<String, io::Error> {
        let full_path = self.joined();
        let mut file = OpenOptions::new().read(true).open(full_path)?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        Ok(buffer)
    }

    pub fn write_to_file(&self, contents: &[u8]) -> Result<(), io::Error> {
        let full_path = self.joined();

        // TODO: Improvement, use .append(true) instead of write.
        let mut file = OpenOptions::new().write(true).open(full_path)?;

        file.write_all(contents)?;

        Ok(())
    }

    pub fn file_is_empty(&self) -> Result<bool, io::Error> {
        let full_path = self.joined();

        let mut file = File::open(full_path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents.is_empty())
    }
}
