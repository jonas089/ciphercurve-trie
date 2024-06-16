use crate::store::types::{Node, Root};
use std::collections::HashMap;
pub struct InMemoryDB {
    pub root: Root,
    pub nodes: HashMap<Vec<u8>, Node>,
}
impl InMemoryDB {
    pub fn insert(&mut self, key: &Vec<u8>, node: Node) {
        self.nodes.insert(key.clone(), node);
    }
    pub fn get(&mut self, key: &Vec<u8>) -> Option<&mut Node> {
        self.nodes.get_mut(key)
    }
}
