use super::db::Database;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type RootHash = Vec<u8>;
pub type NodeHash = Vec<u8>;
pub type Key = Vec<u8>;
pub type Data = Vec<u8>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
    Root(Root),
    Branch(Branch),
    Leaf(Leaf),
}

impl Node {
    pub fn unwrap_as_root(self) -> Result<Root> {
        match self {
            Node::Root(root) => Ok(root),
            _ => bail!("Failed to unwrap as Root"),
        }
    }
    pub fn unwrap_as_branch(self) -> Result<Branch> {
        match self {
            Node::Branch(branch) => Ok(branch),
            _ => bail!("Failed to unwrap as Branch"),
        }
    }
    pub fn unwrap_as_leaf(self) -> Result<Leaf> {
        match self {
            Node::Leaf(leaf) => Ok(leaf),
            _ => bail!("Failed to unwrap as Leaf"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Root {
    pub hash: Option<RootHash>,
    pub left: Option<NodeHash>,
    pub right: Option<NodeHash>,
}

impl Root {
    pub fn empty() -> Self {
        Self {
            hash: None,
            left: None,
            right: None,
        }
    }
    pub fn store(&self, db: &mut dyn Database) {
        db.insert(
            &self
                .hash
                .clone()
                .expect("Must compute hash before storing a node, try calling .hash()"),
            Node::Root(self.clone()),
        )
    }
    pub fn hash_and_store(&mut self, db: &mut dyn Database) {
        self.hash = None;
        self.hash();
        self.store(db);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Branch {
    pub key: Key,
    pub hash: Option<NodeHash>,
    pub left: Option<NodeHash>,
    pub right: Option<NodeHash>,
}

impl Branch {
    pub fn empty(key: Key) -> Self {
        Self {
            key,
            hash: None,
            left: None,
            right: None,
        }
    }
    pub fn new(key: Key, left: Option<NodeHash>, right: Option<NodeHash>) -> Self {
        Self {
            key,
            hash: None,
            left,
            right,
        }
    }
    pub fn store(&self, db: &mut dyn Database) {
        db.insert(
            &self
                .hash
                .clone()
                .expect("Must compute hash before storing a node, try calling .hash()"),
            Node::Branch(self.clone()),
        )
    }
    pub fn hash_and_store(&mut self, db: &mut dyn Database) {
        self.hash = None;
        self.hash();
        self.store(db);
    }
    pub fn update(&mut self, left: Option<NodeHash>, right: Option<NodeHash>) {
        self.left = left;
        self.right = right;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Leaf {
    pub prefix: Option<Key>,
    pub key: Key,
    pub hash: Option<NodeHash>,
    pub data: Option<Data>,
}

impl Leaf {
    pub fn empty(key: Key) -> Self {
        Self {
            prefix: None,
            key,
            hash: None,
            data: None,
        }
    }
    pub fn new(key: Key, data: Option<Data>) -> Self {
        Self {
            prefix: None,
            key,
            hash: None,
            data,
        }
    }
    pub fn hash_and_store(&mut self, db: &mut dyn Database) {
        self.hash = None;
        self.hash();
        self.store(db);
    }
    pub fn store(&self, db: &mut dyn Database) {
        db.insert(
            &self
                .hash
                .clone()
                .expect("Must compute hash before storing a node, try calling .hash()"),
            Node::Leaf(self.clone()),
        )
    }
}

pub trait Hashable {
    fn hash(&mut self);
}

impl Hashable for Root {
    fn hash(&mut self) {
        self.hash = None;
        self.hash = Some(default_hash(bincode::serialize(&self).unwrap()));
    }
}

impl Hashable for Branch {
    fn hash(&mut self) {
        self.hash = None;
        self.hash = Some(default_hash(bincode::serialize(&self).unwrap()));
    }
}

impl Hashable for Leaf {
    fn hash(&mut self) {
        self.hash = None;
        self.hash = Some(default_hash(bincode::serialize(&self).unwrap()));
    }
}

pub fn default_hash<T: AsRef<[u8]>>(data: T) -> NodeHash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}
