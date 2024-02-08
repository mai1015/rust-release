use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;
use std::time::SystemTime;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use crate::node::diff::FileDiff;
use crate::node::dir::DirectoryNode;

#[derive(Serialize, Deserialize)]
pub struct FileData {
    pub path: String,
    pub version: u64,
    pub time: u64,
    pub root: Option<DirectoryNode>,
}

fn get_time() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

impl Default for FileData {
    fn default() -> Self {
        FileData {
            path: String::new(),
            version: 0,
            time: 0,
            root: None,
        }
    }
}

impl FileData {
    pub fn new(path: String, version: u64, root: DirectoryNode) -> Self {
        FileData {
            path,
            version,
            time: get_time(),
            root: Some(root),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        match File::open(path) {
            Ok(file) => {
                let mut decoder = GzDecoder::new(file);
                let mut bytes = Vec::new();
                decoder.read_to_end(&mut bytes).unwrap();
                let mut file: FileData = bincode::deserialize(&bytes).unwrap_or_default();
                if file.root.as_ref().is_some(){
                    file.root.as_mut().unwrap().restore_path(None);
                }
                file
            },
            Err(_) => FileData::default(),
        }
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let encoded = bincode::serialize(self).expect("Serialization failed");
        let file = File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(&encoded)?;
        encoder.finish()?;
        Ok(())
    }

    pub fn diff(&self, other: &FileData) -> Vec<FileDiff> {
        // if self.version > other.version {
        //     log::warn!("Version is older: source v{} > targe v{}", self.version, other.version);
        // }
        // if self.time != other.time {
        //     diffs.push(format!("Times differ: {} != {}", self.time, other.time));
        // }
        if self.root.is_none() {
            log::error!("Source root is none");
        }
        if other.root.is_none() {
            log::error!("Target root is none");
        }

        self.root.as_ref().unwrap().get_update_list(other.root.as_ref().unwrap())
    }
}