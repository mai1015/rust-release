use std::{fs, io};
use std::path::Path;
use std::sync::Arc;
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

pub fn generate_file_data_from_path<P: AsRef<Path>>(path: P, ignore: &Vec<String>) -> io::Result<FileData> {
    let root = generate_root_from_path(path.as_ref(), "", ignore)?;

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


pub fn generate_root_from_path<P: AsRef<Path>>(path: P, relative_path: &str, ignore: &Vec<String>) -> io::Result<DirectoryNode> {
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