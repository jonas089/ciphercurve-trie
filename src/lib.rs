use store::{
    db::InMemoryDB,
    types::{Branch, Hashable, Key, Leaf, Node, Root},
};

pub mod merkle;
pub mod store;

pub fn insert_leaf(db: &mut InMemoryDB, new_leaf: &mut Leaf, root_node: Node) -> Root {
    assert_eq!(new_leaf.key.len(), 256);
    // maintain a copy of all nodes that must be updated
    // and inserted into the db at their new hashs
    // for each level in the tree at most one node will
    // be added to this list
    let mut new_root: Root = Root::empty();
    let mut modified_nodes: Vec<(u8, Node)> = Vec::new();
    let mut current_node: Node = root_node.clone();
    let mut current_node_pos: u8 = 0;
    let mut idx = 0;
    while idx < new_leaf.key.len() {
        let digit: u8 = new_leaf.key[idx]; // 0 or 1
        assert!(digit == 0 || digit == 1);
        match &mut current_node {
            Node::Root(root) => {
                if digit == 0 {
                    match root.left.clone() {
                        Some(node_hash) => {
                            let root_unwrapped: Root = root_node.clone().unwrap_as_root();
                            modified_nodes.push((0, Node::Root(root_unwrapped)));
                            current_node = db.get(&node_hash).unwrap().clone();
                        }
                        None => {
                            let mut root = current_node.clone().unwrap_as_root();
                            root.left = Some(new_leaf.hash.clone().unwrap());
                            new_leaf.store(db);
                            new_root = root.clone();
                            modified_nodes.push((0, Node::Root(root)));
                            break;
                        }
                    }
                } else {
                    match root.right.clone() {
                        Some(node_hash) => {
                            let root_unwrapped: Root = root_node.clone().unwrap_as_root();
                            modified_nodes.push((0, Node::Root(root_unwrapped)));
                            current_node = db.get(&node_hash).unwrap().clone();
                        }
                        None => {
                            let mut root = current_node.clone().unwrap_as_root();
                            root.right = Some(new_leaf.hash.clone().unwrap());
                            new_leaf.store(db);
                            new_root = root.clone();
                            modified_nodes.push((1, Node::Root(root)));
                            break;
                        }
                    }
                }
            }
            Node::Branch(branch) => {
                idx += branch.key.len();
                if digit == 0 {
                    match branch.left.clone() {
                        Some(node_hash) => {
                            current_node = db.get(&node_hash).unwrap().clone();
                            current_node_pos = 0;
                        }
                        None => {
                            branch.left = Some(new_leaf.hash.clone().unwrap());
                            new_leaf.store(db);
                            // don't do this here, do it when re-hashing the trie.
                            //branch.hash_and_store(db);
                            modified_nodes.push((0, Node::Branch(branch.clone())));
                            break;
                        }
                    }
                } else {
                    match branch.right.clone() {
                        Some(node_hash) => {
                            current_node = db.get(&node_hash).unwrap().clone();
                            current_node_pos = 1;
                        }
                        None => {
                            branch.left = Some(new_leaf.hash.clone().unwrap());
                            new_leaf.store(db);
                            // don't do this here, do it when re-hashing the Trie.
                            //branch.hash_and_store(db);
                            modified_nodes.push((1, Node::Branch(branch.clone())));
                            break;
                        }
                    }
                }
            }
            Node::Leaf(leaf) => {
                let neq_idx = find_key_idx_not_eq(&new_leaf.key[idx..].to_vec(), &leaf.key)
                    .expect("Unhandled Exception");
                let new_leaf_pos: u8 = new_leaf.key[neq_idx];
                // there might be an inefficiency to this?
                // we store leaf again with just a different prefix
                // maybe won't do this in a future release...
                leaf.prefix = Some(leaf.key[neq_idx..].to_vec());
                new_leaf.prefix = Some(new_leaf.key[neq_idx..].to_vec());
                // replace this leaf with a branch in memory
                // re-hashing old leaf here because of prefix change
                leaf.hash_and_store(db);
                // same for new leaf
                new_leaf.hash_and_store(db);
                // don't do this here, do it when re-hashing the Trie
                //new_branch.hash_and_store(db);
                let mut new_branch: Branch = Branch::empty(new_leaf.key[..neq_idx].to_vec());
                if new_leaf_pos == 0 {
                    new_branch.left = new_leaf.hash.clone();
                    new_branch.right = leaf.hash.clone();
                } else {
                    new_branch.left = leaf.hash.clone();
                    new_branch.right = new_leaf.hash.clone();
                }
                modified_nodes.push((current_node_pos, Node::Branch(new_branch)));
                break;
            }
        }
    }
    modified_nodes.reverse();
    for chunk in &mut modified_nodes.chunks(2) {
        if let [child, parent] = chunk {
            // todo: re-hash child and insert it
            // todo: hash child, insert it's hash into the parent and re-hash the parent
            // insert both child and parent into the DB
            let child_node = child.1.clone();
            let child_idx = child.0;
            let parent_node = parent.1.clone();
            match parent_node {
                Node::Root(mut root) => match child_node {
                    Node::Leaf(mut leaf) => {
                        leaf.hash();
                        if child_idx == 0 {
                            root.left = Some(leaf.hash.clone().unwrap());
                        } else {
                            root.right = Some(leaf.hash.clone().unwrap());
                        }
                        leaf.store(db);
                        new_root = root.clone();
                    }
                    Node::Branch(mut branch) => {
                        branch.hash();
                        if child_idx == 0 {
                            root.left = Some(branch.hash.clone().unwrap());
                        } else {
                            root.right = Some(branch.hash.clone().unwrap());
                        }
                        branch.store(db);
                        new_root = root.clone();
                    }
                    _ => panic!("Child can't be a Root"),
                },
                Node::Branch(mut branch) => match child_node {
                    Node::Leaf(mut leaf) => {
                        leaf.hash();
                        if child_idx == 0 {
                            branch.left = Some(leaf.hash.clone().unwrap());
                        } else {
                            branch.right = Some(leaf.hash.clone().unwrap());
                        }
                        leaf.store(db);
                        branch.hash_and_store(db);
                    }
                    Node::Branch(mut branch) => {
                        branch.hash();
                        if child_idx == 0 {
                            branch.left = Some(branch.hash.clone().unwrap());
                        } else {
                            branch.right = Some(branch.hash.clone().unwrap());
                        }
                        branch.store(db);
                        branch.hash_and_store(db);
                    }
                    _ => panic!("Child can't be a Root"),
                },
                _ => panic!("Root can't be a child"),
            }
        }
    }
    new_root.hash = None;
    new_root.hash_and_store(db);
    println!("Root: {:?}", &new_root);
    new_root
}

fn find_key_idx_not_eq(k1: &Key, k2: &Key) -> Option<usize> {
    // todo: find the index at which the keys are not equal
    for (idx, digit) in k1.into_iter().enumerate() {
        if digit != &k2[idx] {
            return Some(idx);
        }
    }
    return None;
}

#[cfg(test)]
mod tests {
    use crate::insert_leaf;
    use crate::store::types::{Hashable, Node, Root};
    use crate::store::{db::InMemoryDB, types::Leaf};
    use std::collections::HashMap;

    #[test]
    fn test_insert_leaf() {
        let mut db = InMemoryDB {
            nodes: HashMap::new(),
        };

        let mut leaf_1: Leaf = Leaf::empty(vec![0u8; 256]);

        let mut leaf_2_key: Vec<u8> = vec![0; 253];

        for _i in 0..3 {
            leaf_2_key.push(1);
        }
        let mut leaf_2: Leaf = Leaf::empty(leaf_2_key);

        let mut leaf_3_key: Vec<u8> = vec![0; 253];
        for _i in 0..3 {
            leaf_3_key.push(0);
        }
        let mut leaf_3 = Leaf::empty(leaf_3_key);
        leaf_1.hash();
        leaf_2.hash();
        leaf_3.hash();
        let root: Root = Root::empty();
        let root_node = Node::Root(root);
        let new_root = insert_leaf(&mut db, &mut leaf_1, root_node);
        let _ = insert_leaf(&mut db, &mut leaf_2, Node::Root(new_root));
    }
}
