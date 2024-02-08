use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::node::file::FileNode;
use crate::node::Node;

pub enum FileDiff {
    Add(FileDetail),
    Change(FileDetail),
    Remove(FileDetail)
}

impl fmt::Display for FileDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileDiff::Add(file) => write!(f, "A: {}", file),
            FileDiff::Change(file) => write!(f, "C: {}", file),
            FileDiff::Remove(file) => write!(f, "R: {}", file)
        }
    }
}

pub struct FileDetail {
    pub path: Arc<str>,
    pub name: String,
    pub is_file: bool
}

impl FileDetail {
    pub fn new(path: Arc<str>, name: String, is_file:bool) -> Self {
        FileDetail {
            path,
            name,
            is_file
        }
    }
    
    pub fn from(node: &Node) -> Self {
        match node {
            Node::Directory(dir) => FileDetail::new(dir.path.as_ref().unwrap().clone(), dir.name.clone(), false),
            Node::File(file) => FileDetail::new(file.path.as_ref().unwrap().clone(), file.name.clone(), true)
        }
    }

    pub fn from_file(file: &FileNode) -> Self {
        FileDetail::new(file.path.as_ref().unwrap().clone(), file.name.clone(), true)
    }

    pub fn get_path<T: AsRef<Path>>(&self, base: T) -> PathBuf {
        if self.is_file {
            base.as_ref().join(self.path.as_ref()).join(&self.name)
        } else {
            base.as_ref().join(self.path.as_ref()).join(&self.name)
        }
    }
}

impl fmt::Display for FileDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.path, std::path::MAIN_SEPARATOR, self.name)
    }
}