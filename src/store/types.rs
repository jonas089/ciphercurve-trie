use sha2::{Digest, Sha256};
#[derive(Clone, Debug)]
pub struct Root {
    pub hash: Option<Vec<u8>>,
    pub left_child: Option<Node>,
    pub right_child: Option<Node>,
}
impl Root {
    pub fn hash(&mut self) {
        let mut preimage: Vec<u8> = Vec::new();
        match &self.left_child {
            Some(child) => match child.clone() {
                Node::Branch(branch) => {
                    preimage.append(&mut branch.hash.unwrap());
                }
                Node::Leaf(leaf) => {
                    preimage.append(&mut leaf.hash.expect("Encountered Leaf with no hash!"));
                }
            },
            None => {
                preimage.push(0u8);
            }
        };
        match &self.right_child {
            Some(child) => match child.clone() {
                Node::Branch(branch) => {
                    preimage.append(&mut branch.hash.unwrap());
                }
                Node::Leaf(leaf) => {
                    preimage.append(&mut leaf.hash.expect("Encountered Leaf with no hash!"));
                }
            },
            None => {
                preimage.push(1u8);
            }
        };
        self.hash = Some(default_hash(&preimage));
    }
}

#[derive(Clone, Debug)]
pub enum Node {
    Branch(Branch),
    Leaf(Leaf),
}

#[derive(Clone, Debug)]
pub struct Leaf {
    pub hash: Option<Vec<u8>>,
    pub key: Vec<u8>,
    pub data: String,
}
impl Leaf {
    // temporary hash function, later implement proper Hasher
    pub fn hash(&mut self) {
        self.hash = Some(default_hash(&self.data))
    }
}

#[derive(Clone, Debug)]
pub struct Branch {
    pub hash: Option<Vec<u8>>,
    pub key: Vec<u8>,
    pub left_child: Option<Box<Node>>,
    pub right_child: Option<Box<Node>>,
}

impl Branch {
    // temporary hash function, later implement proper Hasher
    pub fn hash(&mut self) {
        let mut preimage: Vec<u8> = Vec::new();
        match &self.left_child {
            Some(child) => match *child.clone() {
                Node::Branch(branch) => {
                    preimage.append(&mut branch.hash.unwrap());
                }
                Node::Leaf(leaf) => {
                    preimage.append(&mut leaf.hash.expect("Encountered Leaf with no hash!"));
                }
            },
            None => {
                preimage.push(0u8);
            }
        };
        match &self.right_child {
            Some(child) => match *child.clone() {
                Node::Branch(branch) => {
                    preimage.append(&mut branch.hash.unwrap());
                }
                Node::Leaf(leaf) => {
                    preimage.append(&mut leaf.hash.expect("Encountered Leaf with no hash!"));
                }
            },
            None => {
                preimage.push(1u8);
            }
        };
        self.hash = Some(default_hash(&preimage));
    }
}

pub fn default_hash<T: AsRef<[u8]>>(data: T) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}
