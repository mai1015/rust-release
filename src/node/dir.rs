use std::cmp::Ordering;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::node::diff::{FileDetail, FileDiff};
use crate::node::Node;

#[derive(Clone, Deserialize, Serialize)]
pub struct DirectoryNode {
    #[serde(skip)]
    pub path: Option<Arc<str>>,
    pub name: String,
    pub children: Vec<Node>,
}

impl DirectoryNode {
    pub fn new(name: String, path: Option<Arc<str>>) -> DirectoryNode {
        DirectoryNode {
            path,
            name,
            children: Vec::new(),
        }
    }

    pub fn has_child(&self, child: &Node) -> bool {
        match self.children.binary_search(child) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn remove_child(&mut self, child: &Node) -> Option<Node> {
        match self.children.binary_search(child) {
            Ok(index) => Some(self.children.remove(index)),
            Err(_) => None,
        }
    }

    pub fn add_child(&mut self, child: Node) -> Option<usize> {
        match self.children.binary_search(&child) {
            Ok(_) => None,
            Err(index) => {
                self.children.insert(index, child);
                Some(index)
            }
        }
    }

    pub fn get_path(&self) -> String {
        match self.path {
            None => {
                format!("{}", self.name)
            }
            Some(_) => {
                format!("{}{}{}", self.path.as_ref().unwrap(), std::path::MAIN_SEPARATOR, self.name)
            }
        }
    }

    pub fn needs_update(&self, other: &Self) -> bool {
        self.children.len() != other.children.len() || self.children.iter().zip(other.children.iter()).any(|(a, b)| !a.eq(b) || a.needs_update(b))
    }

    pub fn get_update_list(&self, other: &Self) -> Vec<FileDiff> {
        log::debug!("Checking for: {}", self.get_path());
        let path: Arc<str> = Arc::from(self.get_path());

        let mut update_list = Vec::new();
        let mut i = 0; let mut j = 0;
        while i < self.children.len() && j < other.children.len() {
            let a = &self.children[i];
            let b = &other.children[j];
            match a.cmp(b) {
                Ordering::Less => {
                    update_list.push(FileDiff::Remove(FileDetail::new(path.clone(), a.name().clone(), a.is_file())));
                    i += 1;
                }
                Ordering::Greater => {
                    update_list.push(FileDiff::Add(FileDetail::new(path.clone(), b.name().clone(), b.is_file())));
                    j += 1;
                }
                Ordering::Equal => {
                    update_list.extend(a.get_update_list(b));
                    i += 1;
                    j += 1;
                }
            }
        }

        if i < self.children.len() {
            for a in &self.children[i..] {
                update_list.push(FileDiff::Remove(FileDetail::new(path.clone(), a.name().clone(), a.is_file())));
            }
        }

        if j < other.children.len() {
            for b in &other.children[j..] {
                match b {
                    Node::Directory(dir) => {
                        update_list.extend(dir.as_add());
                    }
                    Node::File(file) => {
                        update_list.push(FileDiff::Add(FileDetail::new(path.clone(), file.name.clone(), true)));
                    }
                }
            }
        }
        update_list
    }

    fn as_add(&self) -> Vec<FileDiff> {
        let path: Arc<str> = Arc::from(self.get_path());

        let mut update_list = Vec::new();
        for child in &self.children {
            match child {
                Node::Directory(dir) => {
                    update_list.push(FileDiff::Add(FileDetail::new(path.clone(), dir.name.clone(), false)));
                    update_list.extend(dir.as_add());
                }
                Node::File(file) => {
                    update_list.push(FileDiff::Add(FileDetail::new(path.clone(), file.name.clone(), true)));
                }
            }
        }
        update_list
    }

    pub fn restore_path(&mut self, path: Option<Arc<str>>) {
        self.path = path.clone();
        // if it is root as the path is None
        let path: Arc<str> = Arc::from(self.get_path());

        for child in &mut self.children {
            child.restore_path(Some(path.clone()));
        }
    }

    pub fn children(&self) -> &Vec<Node> {
        &self.children
    }
}

impl PartialEq for DirectoryNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for DirectoryNode {}

impl PartialOrd for DirectoryNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DirectoryNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}