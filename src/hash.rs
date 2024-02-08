use std::io::Read;
use std::path::Path;
use sha2::{Sha256, Digest};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub(crate) fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let hash = hash_file(&mut file)?;
    Ok(hash)
}

pub(crate) fn hash_file<R: Read>(rd: &mut R) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];
    loop {
        let n = rd.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}