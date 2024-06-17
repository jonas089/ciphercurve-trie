// Compute Merkle Proof for a Leaf at a given point in time (e.g. at a Snapshot)
use crate::default_hash;
use crate::store::db::InMemoryDB;
use crate::store::types::Node;
// obtain the merkle path for a leaf
pub fn merkle_proof(
    db: &mut InMemoryDB,
    key: Vec<u8>,
) -> (Vec<(Option<Box<Node>>, bool)>, Option<(Option<Node>, bool)>) {
    assert_eq!(key.len(), 256);
    // 0(false): left, 1(true): right
    let mut siblings: Vec<(Option<Box<Node>>, bool)> = Vec::new();
    // get the parent up to the root and collect all the siblings
    let mut current_idx: Vec<u8> = key.clone();
    let mut parent_idx: Vec<u8> = key.clone();
    parent_idx.pop();

    while parent_idx.len() > 0 {
        let parent = db
            .get(&parent_idx)
            .expect("Failed to get parent for node")
            .to_owned();

        match parent {
            Node::Branch(branch) => {
                if current_idx.last().unwrap() == &0 {
                    siblings.push((branch.right_child, true));
                } else {
                    siblings.push((branch.left_child, false));
                }
                current_idx.pop();
            }
            Node::Leaf(_) => {
                panic!("Leaf can't be Root Branch");
            }
        }
        parent_idx.pop();
    }
    #[allow(unused_assignments)]
    let mut root_sibling: Option<(Option<Node>, bool)> = None;
    if key.get(0).unwrap() == &0 {
        root_sibling = Some((db.root.right_child.clone(), true));
    } else {
        root_sibling = Some((db.root.left_child.clone(), false));
    }
    (siblings, root_sibling)
}

pub fn compute_root(
    merkle_proof: (Vec<(Option<Box<Node>>, bool)>, Option<(Option<Node>, bool)>),
    leaf_hash: Vec<u8>,
) -> Vec<u8> {
    let merkle_path_base = merkle_proof.0.clone();
    let mut current_hash: Vec<u8> = leaf_hash;
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

    let merkle_path_root = merkle_proof.1.unwrap();
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
    root_hash
}
