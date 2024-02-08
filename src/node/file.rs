use std::cmp::Ordering;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::node::diff::{FileDetail, FileDiff};


#[derive(Clone, Deserialize, Serialize)]
pub struct FileNode {
    #[serde(skip)]
    pub path: Option<Arc<str>>,
    pub name: String,
    pub last_modified: u64,
    pub hash: Option<String>,
}

impl FileNode {
    pub fn new(path: Arc<str>, name: String, last_modified: u64) -> Self {
        FileNode {
            path: Some(path),
            name,
            last_modified,
            hash: None,
        }
    }

    pub fn get_path(&self) -> String {
        match self.path {
            None => {
                format!(".{}{}", std::path::MAIN_SEPARATOR, self.name)
            }
            Some(_) => {
                format!("{}{}{}", self.path.as_ref().unwrap(), std::path::MAIN_SEPARATOR, self.name)
            }
        }

    }

    pub fn set_hash(&mut self, hash: String) {
        self.hash = Some(hash);
    }

    pub fn needs_update(&self, other: &Self) -> bool {
        if self.hash.is_some() && other.hash.is_some() {
            self.hash != other.hash
        } else {
            self.last_modified < other.last_modified
        }
    }

    pub fn get_update_list(&self, other: &Self) -> Vec<FileDiff> {
        if self.path.is_none() {
            panic!("Path is none");
        }

        if self.needs_update(other) {
            vec![FileDiff::Change(FileDetail::new(self.path.as_ref().unwrap().clone(), self.name.clone(), true))]
        } else {
            Vec::new()
        }
    }

    pub fn restore_path(&mut self, path: Option<Arc<str>>) {
        if path.is_none() {
            panic!("Path is none");
        }

        self.path = path;
        log::debug!("Restoring path: {}{}{}", self.path.as_ref().unwrap(), std::path::MAIN_SEPARATOR, self.name);
    }
}

impl PartialEq for FileNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for FileNode {}

impl PartialOrd for FileNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::node::file::FileNode;

    #[test]
    fn test_file_node() {
        let path: Arc<str> = Arc::from(String::from("test").as_ref());
        let file = FileNode::new(path, "test".to_string(), 0);
        assert_eq!(file.get_path(), "Some(\"test\")/test");
    }
}
