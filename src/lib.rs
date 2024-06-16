use std::collections::HashMap;

use sha2::{Sha256, Digest};

pub struct InMemoryDB{
    pub root: Root,
    pub nodes: HashMap<Vec<u8>, Node>
}
impl InMemoryDB{
    pub fn insert(&mut self, key: &Vec<u8>, node: Node){
        self.nodes.insert(key.clone(), node);
    }
    pub fn get(&mut self, key: &Vec<u8>) -> Option<&mut Node>{
        self.nodes.get_mut(key)
    }
}

#[derive(Clone, Debug)]
pub struct Root{
    pub hash: Option<Vec<u8>>,
    pub left_child: Option<Node>,
    pub right_child: Option<Node>
}
impl Root{
    pub fn hash(&mut self){
        let mut preimage: Vec<u8> = Vec::new();
        match &self.left_child{
            Some(child) => {
                match child.clone(){
                    Node::Branch(branch) => {
                        preimage.append(&mut branch.hash.unwrap());
                    },
                    Node::Leaf(leaf) => {
                        preimage.append(&mut leaf.hash.expect("Encountered Leaf with no hash!"));
                    }
                }
            },
            None => {
                preimage.push(0u8);
            }
        };
        match &self.right_child{
            Some(child) => {
                match child.clone(){
                    Node::Branch(branch) => {
                        preimage.append(&mut branch.hash.unwrap());
                    },
                    Node::Leaf(leaf) => {
                        preimage.append(&mut leaf.hash.expect("Encountered Leaf with no hash!"));
                    }
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
pub enum Node{
    Branch(Branch),
    Leaf(Leaf)
}

#[derive(Clone, Debug)]
pub struct Leaf{
    pub hash: Option<Vec<u8>>,
    pub key: Vec<u8>,
    pub data: String
}
impl Leaf{
    // temporary hash function, later implement proper Hasher
    pub fn hash(&mut self){
        self.hash = Some(default_hash(&self.data))
    }
}

#[derive(Clone, Debug)]
pub struct Branch{
    pub hash: Option<Vec<u8>>,
    pub key: Vec<u8>,
    pub left_child: Option<Box<Node>>,
    pub right_child: Option<Box<Node>>
}

impl Branch{
    // temporary hash function, later implement proper Hasher
    pub fn hash(&mut self){
        let mut preimage: Vec<u8> = Vec::new();
        match &self.left_child{
            Some(child) => {
                match *child.clone(){
                    Node::Branch(branch) => {
                        preimage.append(&mut branch.hash.unwrap());
                    },
                    Node::Leaf(leaf) => {
                        preimage.append(&mut leaf.hash.expect("Encountered Leaf with no hash!"));
                    }
                }
            },
            None => {
                preimage.push(0u8);
            }
        };
        match &self.right_child{
            Some(child) => {
                match *child.clone(){
                    Node::Branch(branch) => {
                        preimage.append(&mut branch.hash.unwrap());
                    },
                    Node::Leaf(leaf) => {
                        preimage.append(&mut leaf.hash.expect("Encountered Leaf with no hash!"));
                    }
                }
            },
            None => {
                preimage.push(1u8);
            }
        };
        self.hash = Some(default_hash(&preimage));
    }
}

pub fn default_hash<T: AsRef<[u8]>>(data: T) -> Vec<u8>{
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn insert_leaf(db: &mut InMemoryDB, key: Vec<u8>, data: String){
    let mut current_idx: Vec<u8> = Vec::new();
    // store the new root branch that is the left, or the right child of the root
    let mut new_root_branch: Option<Node> = None;
    for (idx, digit) in key.clone().into_iter().enumerate(){
        current_idx.push(digit);
        // commit if we are at the leaf idx
        if idx == key.len()-1{
            // todo: handle collisions
            let mut new_leaf: Leaf = Leaf{
                hash: None,
                key: current_idx.clone(),
                data: data
            };
            new_leaf.hash();

            // find parent and insert leaf as child (LoR depending on idx)
            let mut parent_idx: Vec<u8> = current_idx.clone();
            parent_idx.pop();
            let parent = db.get(&parent_idx).expect("Failed to get parent for node");
            update_parent(parent, Node::Leaf(new_leaf.clone()), digit);
            // store the leaf in the db
            db.insert(&key, Node::Leaf(new_leaf));
            // done inserting
            break;
        }
        // check if the branch exists, if not create it
        match db.get(&current_idx){
            Some(_) => {
                // Since the Branch already exists, we don't need to do anything.
            },
            None => {
                let new_branch = Branch{
                    hash: None,
                    key: current_idx.clone(),
                    left_child: None,
                    right_child: None
                };
                // The Branch does not exist, therefore we must create it.
                // Insert this Branch as a child to its parent.
                let mut parent_idx: Vec<u8> = current_idx.clone();
                parent_idx.pop();
                if current_idx.len() > 1{
                    let parent = db.get(&parent_idx).expect("Failed to get parent for node");
                    update_parent(parent, Node::Branch(new_branch.clone()), digit);
                }
                db.insert(&current_idx, Node::Branch(new_branch));
            }
        }
    }

    let mut hasher_idx: Vec<u8> = current_idx.clone();
    hasher_idx.pop();

    while hasher_idx.len() > 1{
        let mut current_branch: Node = db.get(&hasher_idx).expect("Failed to find parent branch").to_owned();
        match &mut current_branch{
            Node::Branch(branch) => {
                branch.hash();
            },
            Node::Leaf(_) => {
                panic!("Leaf can't be Branch");
            }
        };
        
        db.insert(&hasher_idx, current_branch.clone());
        
        let mut parent_idx: Vec<u8> = hasher_idx.clone();
        parent_idx.pop();
        
        let mut parent = db.get(&parent_idx).expect("Failed to get parent for node").to_owned();
        update_parent(&mut parent, current_branch.clone(), hasher_idx.last().unwrap().to_owned());

        if hasher_idx.len() == 2{
            // todo: hash and insert the parent
            match &mut parent{
                Node::Branch(branch) => {
                    branch.hash();
                },
                Node::Leaf(_) => panic!("Leaf can't be Root Branch")
            }
            new_root_branch = Some(parent.clone());
        };
        db.insert(&parent_idx, parent.clone());
        hasher_idx.pop();
    };

    if key.get(0).unwrap() == &0{
        db.root.left_child = new_root_branch;
    }
    else{
        db.root.right_child = new_root_branch;
    }
    // re-hash the root
    db.root.hash();
}


pub fn update_parent(parent: &mut Node, node: Node, digit: u8){
    match parent{
        Node::Branch(branch) => {
            if digit == 0_u8{
                // insert the new branch as a left child to the parent branch
                branch.left_child = Some(Box::new(node.clone()));
            }
            else{
                // insert the new branch as a right child to the parent branch
                branch.right_child = Some(Box::new(node.clone()));
            }
        },
        Node::Leaf(_) => {
            panic!("Leaf can't be Branch")
        }
    }  
}

#[test]
fn test_insert_leaf(){
    let mut db: InMemoryDB = InMemoryDB { root: Root{
            hash: None,
            left_child: None,
            right_child: None
        }, 
        nodes: HashMap::new()
    };
    let key: Vec<u8> = vec![0u8;256];
    let data: String = "Casper R&D @ Jonas Pauli".to_string();
    insert_leaf(&mut db, key, data);
    println!("Root: {:?}", &db.root.hash);
}