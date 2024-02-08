use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::node::diff::FileDiff;
use crate::node::dir::DirectoryNode;
use crate::node::file::FileNode;

pub mod diff;
pub mod dir;
pub mod file;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Node {
    File(FileNode),
    Directory(DirectoryNode),
}

impl Node {
    pub fn name(&self) -> &String {
        match self {
            Node::File(file) => &file.name,
            Node::Directory(dir) => &dir.name,
        }
    }

    pub fn get_path(&self) -> String {
        match self {
            Node::File(file) => file.get_path(),
            Node::Directory(dir) => dir.get_path(),
        }
    }

    pub fn needs_update(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::File(a), Node::File(b)) => a.needs_update(b),
            (Node::Directory(a), Node::Directory(b)) => a.needs_update(b),
            _ => true,
        }
    }

    pub fn get_update_list(&self, other: &Self) -> Vec<FileDiff> {
        match (self, other) {
            (Node::File(a), Node::File(b)) => a.get_update_list(b),
            (Node::Directory(a), Node::Directory(b)) => a.get_update_list(b),
            _ => panic!("Cannot compare file and directory"),
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Node::File(_) => true,
            _ => false,
        }
    }

    pub fn restore_path(&mut self, path: Option<Arc<str>>) {
        match self {
            Node::File(file) => file.restore_path(path),
            Node::Directory(dir) => dir.restore_path(path),
        }
    }
}