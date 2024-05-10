use std::cmp::Ordering;
use std::sync::Arc;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::node::diff::{FileDetail, FileDiff};


#[derive(Clone, Deserialize, Serialize)]
pub struct FileNode {
    #[serde(skip)]
    pub path: Option<Arc<str>>,
    pub name: String,
    pub last_modified: u64,
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub hash: [char; 64],
}

fn serialize_hash<S>(hash: &[char; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    let hash_string: String = hash.iter().collect();
    serializer.serialize_str(&hash_string)
}

fn deserialize_hash<'de, D>(deserializer: D) -> Result<[char; 64], D::Error>
    where
        D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let mut chars = s.chars();
    let mut hash = [' '; 64]; // initialize with spaces
    for (i, c) in hash.iter_mut().enumerate() {
        *c = chars.next().unwrap_or(' '); // fill with space if not enough chars
    }
    Ok(hash)
}

unsafe impl Sync for FileNode {}
unsafe impl Send for FileNode {}

impl FileNode {
    pub fn new(path: Arc<str>, name: String, last_modified: u64) -> Self {
        FileNode {
            path: Some(path),
            name,
            last_modified,
            hash: [' '; 64],
        }
    }

    pub fn has_hash(&self) -> bool {
        self.hash != [' '; 64]
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
        if hash.len() != 64 {
            panic!("Hash length is not 64");
        }
        let char_vec: Vec<char> = hash.chars().collect();
        self.hash.copy_from_slice(&char_vec[..64])
    }

    pub fn needs_update(&self, other: &Self) -> bool {
        if self.has_hash() && other.has_hash() {
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
    use std::path;
    use std::sync::Arc;
    use crate::node::file::FileNode;

    #[test]
    fn test_file_node() {
        let path: Arc<str> = Arc::from(String::from("test").as_ref());
        let file = FileNode::new(path, "test".to_string(), 0);
        assert_eq!(file.get_path(), format!("test{}test", path::MAIN_SEPARATOR_STR));
    }
}
