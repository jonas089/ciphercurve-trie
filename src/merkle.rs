// Compute Merkle Proof for a Leaf at a given point in time (e.g. at a Snapshot)
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
    println!("Root sibling: {:?}", &root_sibling);
    (siblings, root_sibling)
}
