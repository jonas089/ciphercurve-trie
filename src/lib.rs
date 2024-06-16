pub mod merkle;
pub mod store;
use bincode;
use serde::{Deserialize, Serialize};
use store::db::InMemoryDB;
#[allow(unused_imports)]
use store::types::{default_hash, Branch, Leaf, Node};

pub fn insert_leaf<T>(db: &mut InMemoryDB, key: Vec<u8>, data: T)
where
    T: Serialize + Deserialize<'static>,
{
    assert_eq!(key.len(), 256);
    let mut current_idx: Vec<u8> = Vec::new();
    // store the new root branch that is the left, or the right child of the root
    let mut new_root_branch: Option<Node> = None;
    for (idx, digit) in key.clone().into_iter().enumerate() {
        current_idx.push(digit);
        // commit if we are at the leaf idx
        if idx == key.len() - 1 {
            // todo: handle collisions
            let mut new_leaf: Leaf = Leaf {
                hash: None,
                key: current_idx.clone(),
                data: bincode::serialize(&data).expect("Failed to serialize leaf with Bincode"),
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
        match db.get(&current_idx) {
            Some(_) => {
                // Since the Branch already exists, we don't need to do anything.
            }
            None => {
                let new_branch = Branch {
                    hash: None,
                    key: current_idx.clone(),
                    left_child: None,
                    right_child: None,
                };
                // The Branch does not exist, therefore we must create it.
                // Insert this Branch as a child to its parent.
                let mut parent_idx: Vec<u8> = current_idx.clone();
                parent_idx.pop();
                if current_idx.len() > 1 {
                    let parent = db.get(&parent_idx).expect("Failed to get parent for node");
                    update_parent(parent, Node::Branch(new_branch.clone()), digit);
                }
                db.insert(&current_idx, Node::Branch(new_branch));
            }
        }
    }

    let mut hasher_idx: Vec<u8> = current_idx.clone();
    hasher_idx.pop();
    // if hasher_idx is 1, then it has no parent
    // in better words, the parent is the root sibling
    // which is stored only in the root
    while hasher_idx.len() > 1 {
        let mut current_branch: Node = db
            .get(&hasher_idx)
            .expect("Failed to find parent branch")
            .to_owned();
        match &mut current_branch {
            Node::Branch(branch) => {
                branch.hash();
            }
            Node::Leaf(_) => {
                panic!("Leaf can't be Branch");
            }
        };

        db.insert(&hasher_idx, current_branch.clone());

        let mut parent_idx: Vec<u8> = hasher_idx.clone();
        parent_idx.pop();

        let mut parent = db
            .get(&parent_idx)
            .expect("Failed to get parent for node")
            .to_owned();
        update_parent(
            &mut parent,
            current_branch.clone(),
            hasher_idx.last().unwrap().to_owned(),
        );

        if hasher_idx.len() == 2 {
            match &mut parent {
                Node::Branch(branch) => {
                    branch.hash();
                }
                Node::Leaf(_) => panic!("Leaf can't be Root Branch"),
            }
            new_root_branch = Some(parent.clone());
        };
        db.insert(&parent_idx, parent.clone());
        hasher_idx.pop();
    }

    if key.get(0).unwrap() == &0 {
        db.root.left_child = new_root_branch;
    } else {
        db.root.right_child = new_root_branch;
    }
    db.root.hash();
}

pub fn update_parent(parent: &mut Node, node: Node, digit: u8) {
    match parent {
        Node::Branch(branch) => {
            if digit == 0_u8 {
                // insert the new branch as a left child to the parent branch
                branch.left_child = Some(Box::new(node.clone()));
            } else {
                // insert the new branch as a right child to the parent branch
                branch.right_child = Some(Box::new(node.clone()));
            }
        }
        Node::Leaf(_) => {
            panic!("Leaf can't be Branch")
        }
    }
}

#[test]
fn test_insert_leaf() {
    use crate::merkle::merkle_proof;
    use crate::store::types::Root;
    use std::collections::HashMap;
    let mut db: InMemoryDB = InMemoryDB {
        root: Root {
            hash: None,
            left_child: None,
            right_child: None,
        },
        nodes: HashMap::new(),
    };
    let key: Vec<u8> = vec![0u8; 256];

    let mut key_2: Vec<u8> = vec![0u8; 255];
    key_2.push(1);

    let key_3: Vec<u8> = vec![1u8; 256];

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct AnyDataFits {
        info: String,
    }

    let data: AnyDataFits = AnyDataFits {
        info: "Jonas @ Casper R&D".to_string(),
    };
    let data_2: AnyDataFits = AnyDataFits {
        info: "Tries are incredible!".to_string(),
    };
    let data_3: AnyDataFits = AnyDataFits {
        info: "K.I.Z für Immer!".to_string(),
    };

    insert_leaf(&mut db, key.clone(), data.clone());
    insert_leaf(&mut db, key_2.clone(), data_2.clone());
    insert_leaf(&mut db, key_3.clone(), data_3.clone());

    let merkle_path = merkle_proof(&mut db, key_3.clone());
    let merkle_path_base = merkle_path.0.clone();
    let init_hash: Vec<u8> = db.get(&key_3).unwrap().unwrap_as_leaf().hash.unwrap();
    let mut current_hash: Vec<u8> = init_hash;
    for sibling in merkle_path_base {
        let current_sibling = sibling.0;
        let sibling_hash: Option<Vec<u8>> = match current_sibling {
            Some(sibling) => match *sibling {
                Node::Branch(branch) => Some(branch.hash.unwrap().to_vec()),
                Node::Leaf(leaf) => Some(leaf.hash.unwrap().to_vec()),
            },
            None => None,
        };
        if sibling.1 == false {
            if let Some(mut hash) = sibling_hash {
                hash.append(&mut current_hash);
                current_hash = default_hash(hash);
            } else {
                let mut preimage = vec![0];
                preimage.append(&mut current_hash);
                current_hash = default_hash(preimage);
            }
        } else {
            if let Some(mut hash) = sibling_hash {
                current_hash.append(&mut hash);
                current_hash = default_hash(current_hash);
            } else {
                current_hash.push(1);
                current_hash = default_hash(current_hash);
            }
        }
    }

    let merkle_path_root = merkle_path.1.unwrap();
    #[allow(unused_assignments)]
    let mut root_sibling_hash: Vec<u8> = Vec::new();
    #[allow(unused_assignments)]
    let mut root_hash: Vec<u8> = Vec::new();
    if merkle_path_root.1 == false {
        match merkle_path_root.0 {
            Some(node) => {
                root_sibling_hash = node.unwrap_as_branch().hash.unwrap();
            }
            None => {
                root_sibling_hash = vec![0];
            }
        }
        root_sibling_hash.append(&mut current_hash);
        root_hash = default_hash(root_sibling_hash);
    } else {
        match merkle_path_root.0 {
            Some(node) => {
                root_sibling_hash = node.unwrap_as_branch().hash.unwrap();
            }
            None => {
                root_sibling_hash = vec![1];
            }
        }
        current_hash.append(&mut root_sibling_hash);
        root_hash = default_hash(current_hash);
    }
    assert_eq!(&root_hash, &db.root.hash.unwrap());
}
