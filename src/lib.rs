use store::{
    db::Database,
    types::{Branch, Key, Leaf, Node, Root},
};

pub mod error;
pub mod merkle;
pub mod store;
use anyhow::{bail, Result};

pub fn check_leaf(db: &mut dyn Database, leaf_expected: &Leaf, mut current_node: Node) -> bool {
    #[allow(unused_assignments)]
    let mut result: bool = false;
    loop {
        match &current_node {
            Node::Branch(branch) => {
                let branch_prefix = &branch.key;
                let neq_idx = &branch_prefix[0];
                let child_idx = leaf_expected.key[*neq_idx as usize];
                if child_idx == 0 {
                    current_node = db.get(&branch.left.clone().unwrap()).unwrap().clone();
                } else {
                    current_node = db.get(&branch.right.clone().unwrap()).unwrap().clone();
                }
            }
            Node::Leaf(leaf) => {
                if leaf.hash == leaf_expected.hash {
                    result = true;
                } else {
                    result = false;
                }
                break;
            }
            Node::Root(root) => {
                if leaf_expected.key[0] == 0 {
                    match &root.left {
                        Some(node) => {
                            current_node = db.get(&node).unwrap().clone();
                        }
                        None => {
                            result = false;
                            break;
                        }
                    }
                } else {
                    match &root.right {
                        Some(node) => {
                            current_node = db.get(&node).unwrap().clone();
                        }
                        None => {
                            result = false;
                            break;
                        }
                    }
                }
            }
        }
    }
    result
}

pub fn insert_leaf(db: &mut dyn Database, new_leaf: &mut Leaf, root_node: Node) -> Result<Root> {
    assert_eq!(new_leaf.key.len(), 256);
    // don't insert if a leaf already exists at the given key
    if check_leaf(db, &new_leaf, root_node.clone()) {
        bail!("Leaf already exists!");
    }
    let modified_nodes = traverse_trie(db, new_leaf, root_node.clone(), false)?;
    let mut new_root = update_modified_leafs(db, modified_nodes, root_node.unwrap_as_root()?)?;
    new_root.hash_and_store(db);
    Ok(new_root)
}

fn traverse_trie(
    db: &mut dyn Database,
    new_leaf: &mut Leaf,
    root_node: Node,
    update: bool,
) -> Result<Vec<(u8, Node)>> {
    let mut modified_nodes: Vec<(u8, Node)> = Vec::new();
    let mut current_node: Node = root_node.clone();
    let mut current_node_pos: u8 = 0;
    loop {
        match &mut current_node {
            Node::Root(root) => {
                if new_leaf.key[0] == 0 {
                    match root.left.clone() {
                        Some(node_hash) => {
                            current_node = db.get(&node_hash).unwrap().clone();
                            current_node_pos = 0;
                        }
                        None => {
                            let mut root: Root = current_node.unwrap_as_root()?;
                            root.left = Some(new_leaf.hash.clone().unwrap());
                            new_leaf.store(db);
                            modified_nodes.push((0, Node::Leaf(new_leaf.clone())));
                            break;
                        }
                    }
                } else {
                    match root.right.clone() {
                        Some(node_hash) => {
                            current_node = db.get(&node_hash).unwrap().clone();
                            current_node_pos = 1;
                        }
                        None => {
                            let mut root = current_node.clone().unwrap_as_root()?;
                            root.right = Some(new_leaf.hash.clone().unwrap());
                            new_leaf.store(db);
                            modified_nodes.push((1, Node::Leaf(new_leaf.clone())));
                            break;
                        }
                    }
                }
            }
            Node::Branch(branch) => {
                if new_leaf.key[branch.key[0] as usize] == 0 {
                    match branch.left.clone() {
                        Some(node_hash) => {
                            modified_nodes.push((current_node_pos, Node::Branch(branch.clone())));
                            current_node = db.get(&node_hash).unwrap().clone();
                            current_node_pos = 0;
                        }
                        None => {
                            bail!("A branch must have 2 children")
                        }
                    }
                } else {
                    match branch.right.clone() {
                        Some(node_hash) => {
                            modified_nodes.push((current_node_pos, Node::Branch(branch.clone())));
                            current_node = db.get(&node_hash).unwrap().clone();
                            current_node_pos = 1;
                        }
                        None => {
                            bail!("A branch must have 2 children")
                        }
                    }
                }
            }
            Node::Leaf(leaf) => {
                if !update {
                    let neq_idx = find_key_idx_not_eq(&new_leaf.key, &leaf.key)
                        .expect("Can't insert duplicate Leaf");
                    let new_leaf_pos: u8 = new_leaf.key[neq_idx];
                    match new_leaf.hash {
                        Some(_) => {}
                        None => bail!("Leaf was not hashed!"),
                    }
                    new_leaf.store(db);
                    let mut new_branch: Branch = Branch::empty(vec![neq_idx as u8]);
                    if new_leaf_pos == 0 {
                        new_branch.left = new_leaf.hash.clone();
                        new_branch.right = leaf.hash.clone();
                    } else {
                        new_branch.left = leaf.hash.clone();
                        new_branch.right = new_leaf.hash.clone();
                    }
                    new_branch.hash_and_store(db);
                    modified_nodes.push((current_node_pos, Node::Branch(new_branch)));
                    break;
                } else {
                    todo!("Currently not supported");
                }
            }
        }
    }
    Ok(modified_nodes)
}

fn update_modified_leafs(
    db: &mut dyn Database,
    mut modified_nodes: Vec<(u8, Node)>,
    old_root: Root,
) -> Result<Root> {
    let mut new_root = Root::empty();
    modified_nodes.reverse();
    modified_nodes.push((0, Node::Root(old_root)));
    for i in 1..modified_nodes.len() {
        let child = modified_nodes[i - 1].clone();
        let parent = modified_nodes[i].clone();
        match parent.1 {
            Node::Root(mut root) => match &child.1 {
                Node::Branch(branch) => {
                    if child.0 == 0 {
                        root.left = Some(branch.clone().hash.unwrap());
                        assert!(root.left.is_some());
                        root.hash_and_store(db);
                        new_root = root;
                    } else {
                        root.right = Some(branch.clone().hash.unwrap());
                        assert!(root.right.is_some());
                        root.hash_and_store(db);
                        new_root = root;
                    }
                }
                Node::Leaf(leaf) => {
                    if child.0 == 0 {
                        root.left = Some(leaf.clone().hash.unwrap());
                        assert!(root.left.is_some());
                        root.hash_and_store(db);
                        new_root = root;
                    } else {
                        root.right = Some(leaf.clone().hash.unwrap());
                        assert!(root.right.is_some());
                        root.hash_and_store(db);
                        new_root = root;
                    }
                }
                _ => bail!("This should never happen, child is root"),
            },
            Node::Branch(mut branch) => match &child.1 {
                Node::Branch(child_branch) => {
                    if child.0 == 0 {
                        branch.left = Some(child_branch.clone().hash.unwrap());
                        branch.hash_and_store(db);
                        modified_nodes[i] = (parent.0, Node::Branch(branch.clone()));
                    } else {
                        branch.right = Some(child_branch.clone().hash.unwrap());
                        branch.hash_and_store(db);
                        modified_nodes[i] = (parent.0, Node::Branch(branch.clone()));
                    }
                }
                Node::Leaf(leaf) => {
                    if child.0 == 0 {
                        branch.left = Some(leaf.clone().hash.unwrap());
                        branch.hash_and_store(db);
                        modified_nodes[i] = (parent.0, Node::Branch(branch.clone()));
                    } else {
                        branch.right = Some(leaf.clone().hash.unwrap());
                        branch.hash_and_store(db);
                        modified_nodes[i] = (parent.0, Node::Branch(branch.clone()));
                    }
                }
                _ => bail!("This should never happen, child is leaf"),
            },
            Node::Leaf(_) => bail!("This should never happen, parent is leaf"),
        }
    }
    assert!(new_root.left.is_some() || new_root.right.is_some());
    Ok(new_root)
}

fn find_key_idx_not_eq(k1: &Key, k2: &Key) -> Option<usize> {
    // todo: find the index at which the keys are not equal
    for (idx, digit) in k1.iter().enumerate() {
        if digit != &k2[idx] {
            return Some(idx);
        }
    }
    None
}

#[test]
fn test_find_key_neq() {
    let x = vec![0, 1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 1];
    let y = vec![0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1];
    assert_eq!(find_key_idx_not_eq(&x, &y).unwrap(), 9);
}

#[cfg(test)]
mod tests {
    use crate::merkle::tests::{generate_random_data, generate_random_key};
    use crate::store::db::sql::TrieDB;
    use crate::store::types::Leaf;
    use crate::store::types::{Hashable, Node, Root};
    use crate::{check_leaf, insert_leaf};
    use colored::*;
    use indicatif::ProgressBar;
    use std::env;
    use std::time::Instant;
    #[test]
    fn test_insert_leaf() {
        let start_time = Instant::now();
        let mut db = TrieDB {
            path: env::var("PATH_TO_DB").unwrap_or("database.sqlite".to_string()),
            cache: None,
        };
        db.setup();
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
        let mut new_root = insert_leaf(&mut db, &mut leaf_1, root_node.clone()).unwrap();
        new_root = insert_leaf(&mut db, &mut leaf_2, Node::Root(new_root)).unwrap();

        assert!(check_leaf(&mut db, &leaf_1, Node::Root(new_root.clone())));
        assert!(check_leaf(&mut db, &leaf_2, Node::Root(new_root.clone())));

        println!(
            "{} Elapsed Time: {} µs",
            "[1x Insert]".yellow(),
            &start_time.elapsed().as_micros().to_string().blue()
        );
    }

    #[test]
    fn test_many_leafs() {
        let transaction_count: u32 = std::env::var("INSERT_TRANSACTION_COUNT")
            .unwrap_or_else(|_| "10000".to_string())
            .parse::<u32>()
            .expect("Invalid argument STRESS_TEST_TRANSACTION_COUNT");
        let mut transactions: Vec<Leaf> = Vec::new();
        for _ in 0..transaction_count {
            let leaf_key = generate_random_key();
            let leaf: Leaf = Leaf::new(leaf_key, Some(generate_random_data()));
            transactions.push(leaf);
        }
        let start_time = Instant::now();
        let mut db = TrieDB {
            path: env::var("PATH_TO_DB").unwrap_or("database.sqlite".to_string()),
            cache: None,
        };
        let root: Root = Root::empty();
        let mut root_node = Node::Root(root);
        let progress_bar: ProgressBar = ProgressBar::new(transaction_count as u64);
        for mut leaf in transactions {
            leaf.hash();
            let new_root = insert_leaf(&mut db, &mut leaf, root_node.clone()).unwrap();
            assert!(check_leaf(
                &mut db,
                &leaf.clone(),
                Node::Root(new_root.clone())
            ));
            root_node = Node::Root(new_root.clone());
            progress_bar.inc(1);
        }
        progress_bar.finish_with_message("Done testing insert!");
        println!(
            "[{}x Insert] Elapsed Time: {} s",
            transaction_count.to_string().yellow(),
            &start_time.elapsed().as_secs().to_string().blue()
        );
    }

    #[test]
    fn test_sql_db() {
        use crate::merkle::{merkle_proof, verify_merkle_proof};
        use crate::store::db::sql::TrieDB;

        let start_time = Instant::now();
        let mut db = TrieDB {
            path: env::var("PATH_TO_DB").unwrap_or("database.sqlite".to_string()),
            cache: None,
        };
        db.setup();
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
        let new_root = insert_leaf(&mut db, &mut leaf_2, Node::Root(new_root.unwrap()));
        println!(
            "{} Elapsed Time: {} µs",
            "[1x Insert]".yellow(),
            &start_time.elapsed().as_micros().to_string().blue()
        );
        let mut leaf_1: Leaf = Leaf::empty(vec![1u8; 256]);
        leaf_1.hash();
        let root: Root = new_root.unwrap();
        let root_node: Node = Node::Root(root);
        let new_root: Root = insert_leaf(&mut db, &mut leaf_1, root_node).unwrap();
        let proof = merkle_proof(&mut db, leaf_1.key, Node::Root(new_root.clone()));
        let inner_proof = proof.unwrap().nodes;
        verify_merkle_proof(inner_proof, new_root.hash.clone().unwrap()).unwrap();
    }
}
