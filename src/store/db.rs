use crate::store::types::Node;
use std::collections::HashMap;

#[derive(Debug)]
pub struct InMemoryDB {
    pub nodes: HashMap<Vec<u8>, Node>,
}
impl InMemoryDB {
    pub fn insert(&mut self, key: &[u8], node: Node) {
        self.nodes.insert(key.to_vec(), node);
    }
    pub fn get(&mut self, key: &Vec<u8>) -> Option<&mut Node> {
        self.nodes.get_mut(key)
    }
}
