use std::path::Path;
use sha2::{Sha256, Digest};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "async")]
use tokio::fs::File;

#[cfg(not(feature = "async"))]
use std::fs::File;

#[cfg(feature = "async")]
use tokio::io::AsyncReadExt;

#[cfg(not(feature = "async"))]
use std::io::Read;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(feature = "async")]
pub async fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path).await?;
    let hash = hash_file(&mut file).await?;
    Ok(hash)
}

#[cfg(not(feature = "async"))]
pub fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let hash = hash_file(&mut file)?;
    Ok(hash)
}

#[cfg(feature = "async")]
async fn hash_file<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];
    loop {
        let n = rd.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(not(feature = "async"))]
fn hash_file<R: Read>(rd: &mut R) -> Result<String> {
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