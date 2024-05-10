
#[cfg(feature = "async")]
use tokio::{io};
#[cfg(feature = "async")]
use async_recursion::async_recursion;

// use tokio::sync::mpsc::Sender;

#[cfg(not(feature = "async"))]
use std::{io};

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::SystemTime;
use crate::data::FileData;
use crate::hash::calculate_file_hash;
use crate::node::dir::DirectoryNode;
use crate::node::file::FileNode;
use crate::node::Node;

// pub type Result<T> = std::result::Result<T, crate::hash::Error>;
//
// use thiserror::Error;
// #[derive(Error, Debug)]
// pub enum Error {
//     #[error("IO error: {0}")]
//     Io(#[from] std::io::Error),
// }

#[cfg(feature = "async")]
#[derive(Debug)]
struct HashData {
    path: String,
    hash: String,
}

#[cfg(feature = "async")]
pub async fn generate_file_data_from_path<P: AsRef<Path>>(path: P, ignore: &Vec<String>) -> io::Result<FileData> {
    let counter = Arc::new(AtomicUsize::new(0));
    // let (tx, mut rx) = mpsc::channel(64);
    // calculate time running 
    let start = SystemTime::now();
    let root = generate_root_from_path(path.as_ref(), "", ignore, counter.clone()).await?;
    log::info!("Scan Done Elapsed time: {}s", SystemTime::now().duration_since(start).unwrap().as_secs());
    loop {
        if counter.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    log::info!("Async Elapsed time: {}s", SystemTime::now().duration_since(start).unwrap().as_secs());
    
    let data = FileData::new(
        path.as_ref().to_str().unwrap().to_string(),
        0,
        root,
    );
    return Ok(data);
}


#[cfg(not(feature = "async"))]
pub fn generate_file_data_from_path<P: AsRef<Path>>(path: P, ignore: &Vec<String>) -> io::Result<FileData> {
    // calculate time running 
    let start = SystemTime::now();
    let root = generate_root_from_path(path.as_ref(), "", ignore)?;
    let elapsed = SystemTime::now().duration_since(start).unwrap().as_secs();
    log::info!("Elapsed time: {}s", elapsed);

    let data = FileData::new(
        path.as_ref().to_str().unwrap().to_string(),
        0,
        root,
    );
    return Ok(data);
}

fn join_path(path: &str, name: &str) -> String {
    if path.is_empty() {
        return name.to_string();
    }
    return format!("{}{}{}", path, std::path::MAIN_SEPARATOR, name);
}

#[cfg(feature = "async")]
struct SafeFileNodePtr(*mut FileNode);

#[cfg(feature = "async")]
unsafe impl Send for SafeFileNodePtr {}

//, tx: Sender<HashData>
#[cfg(feature = "async")]
#[async_recursion]
pub async fn generate_root_from_path<P: AsRef<Path> + std::marker::Send>(path: P, relative_path: &str, ignore: &Vec<String>, counter: Arc<AtomicUsize>) -> io::Result<DirectoryNode> {
    log::debug!("Generate from: {}", path.as_ref().to_str().unwrap_or_default());
    let mut data = if relative_path == "" {
        DirectoryNode::new(
            ".".to_string(),
            None
        )
    } else {
        DirectoryNode::new(
            path.as_ref().file_name().unwrap().to_str().unwrap().to_string(),
            Some(Arc::from(relative_path)),
        )
    };

    let rp = Arc::from(join_path(relative_path, &data.name));

    // initialize with capacity
    let count = fs::read_dir(&path).unwrap().count();
    data.with_capacity(count);

    let paths = fs::read_dir(path).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let path_str = path.to_str().unwrap();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if path.is_dir() {
            let rp_folder = join_path(&rp, &name);
            let should_ignore = ignore.iter().any(|i| {
                rp_folder.starts_with(i) || rp_folder.starts_with(&format!(".{}{}", std::path::MAIN_SEPARATOR, i))
            });
            if should_ignore {
                continue;
            }
            data.add_child(Node::Directory(generate_root_from_path(path_str, &rp, ignore, counter.clone()).await?));
        } else {
            let metadata = fs::metadata(&path)?;
            let last_modified = metadata.modified()?.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let mut file_data = FileNode::new(
                rp.clone(),
                name,
                last_modified,
            );

            let counter_clone = counter.clone();
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            let ptr = SafeFileNodePtr(&mut file_data as *mut FileNode);
            tokio::spawn(async move {
                let hash = calculate_file_hash(&path).await.unwrap();
                let p = ptr;
                // log::debug!("Calculated hash for: {}", path.to_str().unwrap_or_default());
                // tokio::time::sleep(std::time::Duration::from_millis(10));
                unsafe {
                    (*p.0).set_hash(hash);
                }
                log::debug!("Setting hash for: {}", path.to_str().unwrap_or_default());
                counter_clone.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            });
            data.add_child(Node::File(file_data));
        }
    }
    return Ok(data);
}


#[cfg(not(feature = "async"))]
pub fn generate_root_from_path<P: AsRef<Path>>(path: P, relative_path: &str, ignore: &Vec<String>) -> io::Result<DirectoryNode> {
    log::debug!("Generate from: {}", path.as_ref().to_str().unwrap_or_default());
    let mut data = if relative_path == "" {
        DirectoryNode::new(
            ".".to_string(),
            None
        )
    } else {
        DirectoryNode::new(
            path.as_ref().file_name().unwrap().to_str().unwrap().to_string(),
            Some(Arc::from(relative_path)),
        )
    };

    let rp = Arc::from(join_path(relative_path, &data.name));

    let paths = fs::read_dir(path).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let path_str = path.to_str().unwrap();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if path.is_dir() {
            let rp_folder = join_path(&rp, &name);
            let should_ignore = ignore.iter().any(|i| {
                rp_folder.starts_with(i) || rp_folder.starts_with(&format!(".{}{}", std::path::MAIN_SEPARATOR, i))
            });
            if should_ignore {
                continue;
            }
            data.add_child(Node::Directory(generate_root_from_path(path_str, &rp, ignore)?));
        } else {
            let metadata = fs::metadata(&path)?;
            let last_modified = metadata.modified()?.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let mut file_data = FileNode::new(
                rp.clone(),
                name,
                last_modified,
            );
            let hash = calculate_file_hash(&path).unwrap();
            file_data.set_hash(hash);
            data.add_child(Node::File(file_data));
        }
    }

    return Ok(data);
}