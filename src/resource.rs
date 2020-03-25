use crypto::digest::Digest;
use crypto::sha1::Sha1;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::BTreeSet;
use std::cmp::Ordering;
use std::cell::RefCell;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resource {
    pub tags: RefCell<BTreeSet<String>>,
    pub content: String,
    pub hash: String,
}

impl Resource {
    pub(crate) fn new(tags: BTreeSet<String>, content: String) -> Resource {
        let mut hasher = Sha1::new();
        hasher.input_str(&content);
        let hash = hasher.result_str();
        Resource {
            tags: RefCell::new(tags),
            content,
            hash,
        }
    }

    pub fn rm_tag(&mut self, tag: &str) -> Result<(), String> {
        self.tags.get_mut().remove(&tag.to_string());
        Ok(())
    }
}

impl PartialOrd for Resource {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.hash.partial_cmp(&other.hash)
    }
}

impl Ord for Resource {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Resource {}
