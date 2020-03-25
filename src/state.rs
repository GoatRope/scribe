use crate::resource::Resource;
use serde_json::to_string;
use std::cell::RefCell;
use std::clone::Clone;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::iter::Iterator;
use std::path::Path;
use std::{fs, io};
use walkdir::WalkDir;

pub struct State {
    // todo: we want to be able to change the directory at some point
    pub dir: String,
    pub resource_lookup: RefCell<BTreeMap<String, Resource>>,
    pub tag_cache: RefCell<BTreeMap<String, BTreeSet<String>>>,
    pub search_indices: RefCell<BTreeMap<String, BTreeSet<String>>>,
}

impl State {
    pub fn new(dir: String) -> State {
        State {
            dir,
            resource_lookup: RefCell::new(BTreeMap::new()),
            tag_cache: RefCell::new(BTreeMap::new()),
            search_indices: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn initialize(&mut self) -> Result<(), io::Error> {
        self.collect_fs_resources()
            .unwrap()
            .into_iter()
            .for_each(|mut r| {
                self.add_resource(&mut r, false).unwrap();
            });
        Ok(())
    }

    pub fn collect_fs_resources(&mut self) -> Result<Vec<Resource>, io::Error> {
        let mut return_val: Vec<Resource> = Vec::new();
        for result in WalkDir::new(&self.dir).contents_first(true) {
            if let Ok(entry) = result {
                if let Some(resource) = self.new_from_path(entry.path()) {
                    return_val.push(resource);
                }
            }
        }
        Ok(return_val)
    }

    fn new_from_path(&mut self, path: &Path) -> Option<Resource> {
        if path.to_str().unwrap().ends_with("json") {
            let json = read_to_string(&path).expect("read_to_string()");
            let resource: Resource = serde_json::from_str(&json).unwrap();
            return Some(resource);
        }
        None
    }

    pub(crate) fn add_resource(&self, resource: &Resource, sync_to_fs: bool) -> Result<(), String> {
        // see if we've previously cached
        if self.resource_lookup.borrow().contains_key(&resource.hash) {
            self.rm_resource(&resource.hash);
        }
        // set the hash lookup
        self.resource_lookup
            .borrow_mut()
            .insert(resource.hash.clone(), resource.clone());
        // for each tag cache the resource
        for tag in resource.tags.borrow().iter() {
            self.tag_cache
                .borrow_mut()
                .entry(tag.clone())
                .or_insert_with(BTreeSet::<String>::new)
                .insert(resource.hash.clone());
        }
        self.index_resource(resource);
        if sync_to_fs {
            self.sync_resource_to_fs(resource).unwrap();
        }
        Ok(())
    }

    fn sync_resource_to_fs(&self, resource: &Resource) -> std::io::Result<()> {
        let name = format!("{}.json", resource.hash);
        let filename = format!("{}/{}", &self.dir, name);
        if resource.tags.borrow().is_empty() {
            fs::remove_file(filename).unwrap();
        } else {
            let json = serde_json::to_string(&resource).unwrap();
            let mut file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .truncate(true)
                .open(filename)
                .unwrap();
            file.write_all(&json.as_bytes()).unwrap();
        }
        Ok(())
    }

    pub fn index_resource(&self, resource: &Resource) -> () {
        for word in self.split_words(&resource.content).iter() {
            let mut indices = self.search_indices.borrow_mut();
            match indices.get_mut(word) {
                None => {
                    let mut bt = BTreeSet::<String>::new();
                    bt.insert(resource.hash.clone());
                    indices.insert(word.clone(), bt);
                }
                Some(bt) => {
                    bt.insert(resource.hash.clone());
                }
            }
        }
    }

    fn split_words(&self, content: &str) -> Vec<String> {
        fn is_punct(c: char) -> bool {
            // keep underscore, apostrophe, colon
            match c {
                '!'..='&' | '('..='/' | ';'..='@' | '['..='^' | '`' | '{'..='~' => true,
                _ => false,
            }
        }
        content
            .split_whitespace()
            .flat_map(|w| w.split(is_punct))
            .flat_map(to_string)
            .map(|w| w.to_lowercase())
            .map(|w| w.replace("\"", "")) // todo: locate the source of the extra "
            .collect()
    }

    pub fn rm_tag(&self, tag: &str) {
        {
            let mut cache = self.tag_cache.borrow_mut();
            let mut lookup = self.resource_lookup.borrow_mut();
            let set = cache.get_mut(tag).unwrap();
            set.iter()
                .for_each(|hash| lookup.get_mut(hash).unwrap().rm_tag(tag).unwrap());
            cache.remove(tag).unwrap();
        }
        self.index_all_resources();
        self.sync_to_fs();
    }

    pub fn rm_resource(&self, resource_hash: &str) {
        let resource = match self.resource_lookup.borrow_mut().remove(resource_hash) {
            None => return,
            Some(resource) => resource,
        };
        for tag in resource.tags.borrow_mut().iter() {
            self.tag_cache
                .borrow_mut()
                .get_mut(tag)
                .unwrap()
                .remove(&resource.hash);
        }
        self.index_all_resources();
        self.sync_to_fs();
    }

    pub fn sync_to_fs(&self) {
        for resource in self.resource_lookup.borrow().values() {
            self.sync_resource_to_fs(resource).unwrap();
        }
    }

    pub fn index_all_resources(&self) {
        let lookup = self.resource_lookup.borrow();
        self.search_indices.borrow_mut().clear();
        for resource in lookup.values() {
            self.index_resource(resource);
        }
    }
}
