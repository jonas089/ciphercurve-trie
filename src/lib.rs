use store::{
    db::Database,
    types::{Branch, Hashable, Key, Leaf, Node, Root},
};

pub mod merkle;
pub mod store;

pub fn insert_leaf(db: &mut dyn Database, new_leaf: &mut Leaf, root_node: Node) -> Root {
    assert_eq!(new_leaf.key.len(), 256);
    let (modified_nodes, new_root) = traverse_trie(db, new_leaf, root_node, false);
    let mut new_root = update_modified_leafs(db, modified_nodes, new_root);
    new_root.hash_and_store(db);
    new_root
}

pub fn update_leaf(db: &mut dyn Database, new_leaf: &mut Leaf, root_node: Node) -> Root {
    let (modified_nodes, new_root) = traverse_trie(db, new_leaf, root_node, true);
    let mut new_root = update_modified_leafs(db, modified_nodes, new_root);
    new_root.hash_and_store(db);
    new_root
}

pub fn check_leaf(db: &mut dyn Database, leaf_expected: Leaf, root_node: Node) -> bool {
    let mut idx = 0;
    let mut current_node = root_node;
    while idx < leaf_expected.key.len() {
        let digit: u8 = leaf_expected.key[idx];
        match current_node {
            Node::Root(root) => {
                if digit == 0 {
                    match &root.left {
                        Some(node_hash) => {
                            current_node = db.get(&node_hash).unwrap().clone();
                        }
                        None => {
                            println!("No left Child in Root");
                            return false;
                        }
                    }
                } else {
                    match &root.right {
                        Some(node_hash) => {
                            current_node = db.get(&node_hash).unwrap().clone();
                        }
                        None => {
                            println!("No right Child in Root");
                            return false;
                        }
                    }
                }
            }
            Node::Branch(branch) => {
                if digit == 0 {
                    match &branch.left {
                        Some(node_hash) => {
                            current_node = db.get(&node_hash).unwrap().clone();
                        }
                        None => {
                            println!("No left Child in Branch");
                            return false;
                        }
                    }
                } else {
                    match &branch.right {
                        Some(node_hash) => {
                            current_node = db.get(&node_hash).unwrap().clone();
                        }
                        None => {
                            println!("No right Child in Branch");
                            return false;
                        }
                    }
                }
            }
            Node::Leaf(ref leaf) => {
                println!("Leaf left: {:?}, leaf right: {:?}", &leaf, leaf_expected);
                if leaf.data != leaf_expected.data {
                    return false;
                }
            }
        }
        idx += 1;
    }
    true
}

fn traverse_trie(
    db: &mut dyn Database,
    new_leaf: &mut Leaf,
    root_node: Node,
    update: bool,
) -> (Vec<(u8, Node)>, Root) {
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
                            current_node_pos = 0;
                        }
                        None => {
                            let mut root: Root = current_node.unwrap_as_root();
                            root.left = Some(new_leaf.hash.clone().unwrap());
                            new_root = root.clone();
                            new_leaf.store(db);
                            modified_nodes.push((0, Node::Root(root)));
                            modified_nodes.push((0, Node::Leaf(new_leaf.clone())));
                            break;
                        }
                    }
                } else {
                    match root.right.clone() {
                        Some(node_hash) => {
                            let root_unwrapped: Root = root_node.clone().unwrap_as_root();
                            modified_nodes.push((0, Node::Root(root_unwrapped)));
                            current_node = db.get(&node_hash).unwrap().clone();
                            current_node_pos = 1;
                        }
                        None => {
                            let mut root = current_node.clone().unwrap_as_root();
                            root.right = Some(new_leaf.hash.clone().unwrap());
                            new_root = root.clone();
                            new_leaf.store(db);
                            modified_nodes.push((0, Node::Root(root)));
                            modified_nodes.push((1, Node::Leaf(new_leaf.clone())));
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
                            modified_nodes.push((current_node_pos, Node::Branch(branch.clone())));
                            current_node = db.get(&node_hash).unwrap().clone();
                            current_node_pos = 0;
                        }
                        None => {
                            branch.left = Some(new_leaf.hash.clone().unwrap());
                            modified_nodes.push((0, Node::Branch(branch.clone())));
                            break;
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
                            branch.left = Some(new_leaf.hash.clone().unwrap());
                            modified_nodes.push((1, Node::Branch(branch.clone())));
                            break;
                        }
                    }
                }
            }
            Node::Leaf(leaf) => {
                if !update {
                    let neq_idx = find_key_idx_not_eq(&new_leaf.key[idx..].to_vec(), &leaf.key)
                        .expect("Can't insert duplicate Leaf");
                    let new_leaf_pos: u8 = new_leaf.key[neq_idx];
                    // there might be an inefficiency to this?
                    // we store leaf again with just a different prefix
                    // maybe don't do this in a future release...
                    leaf.prefix = Some(leaf.key[neq_idx..].to_vec());
                    new_leaf.prefix = Some(new_leaf.key[neq_idx..].to_vec());
                    leaf.hash_and_store(db);
                    new_leaf.store(db);
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
                } else {
                    todo!("Currently not supported");
                    /*if find_key_idx_not_eq(&new_leaf.key[idx..].to_vec(), &leaf.key).is_some() {
                        panic!("Can't update Leaf since it does not exist");
                    }
                    new_leaf.prefix = leaf.prefix.clone();
                    let new_branch = modified_nodes
                        .last()
                        .expect("Leaf must have Branch or Root above it")
                        .clone();
                    match new_branch.1 {
                        Node::Root(mut root) => {
                            if current_node_pos == 0 {
                                root.left = Some(new_leaf.hash.clone().unwrap());
                            } else {
                                root.right = Some(new_leaf.hash.clone().unwrap());
                            }
                        }
                        Node::Branch(mut branch) => {
                            if current_node_pos == 0 {
                                branch.left = Some(new_leaf.hash.clone().unwrap());
                            } else {
                                branch.right = Some(new_leaf.hash.clone().unwrap());
                            }
                        }
                        _ => panic!("Parent must be Branch or Root"),
                    };
                    break;*/
                }
            }
        }
    }
    (modified_nodes, new_root)
}

fn update_modified_leafs(
    db: &mut dyn Database,
    mut modified_nodes: Vec<(u8, Node)>,
    mut new_root: Root,
) -> Root {
    modified_nodes.reverse();
    for chunk in &mut modified_nodes.chunks(2) {
        if let [child, parent] = chunk {
            println!("chunk: {:?}", &chunk);
            // todo: re-hash child and insert it
            // todo: hash child, insert it's hash into the parent and re-hash the parent
            // insert both child and parent into the DB
            let child_node: Node = child.1.clone();
            let parent_node = parent.1.clone();
            let child_idx = child.0;
            match parent_node {
                Node::Root(mut root) => match child_node {
                    Node::Leaf(leaf) => {
                        if child_idx == 0 {
                            root.left = Some(leaf.hash.clone().unwrap());
                        } else {
                            root.right = Some(leaf.hash.clone().unwrap());
                        }
                        new_root = root.clone();
                    }
                    Node::Branch(mut branch) => {
                        branch.hash_and_store(db);
                        if child_idx == 0 {
                            root.left = Some(branch.hash.clone().unwrap());
                        } else {
                            root.right = Some(branch.hash.clone().unwrap());
                        }
                        new_root = root.clone();
                    }
                    _ => panic!("Child can't be a Root"),
                },
                Node::Branch(mut branch) => {
                    if child_idx == 0 {
                        branch.left = Some(branch.hash.clone().unwrap());
                    } else {
                        branch.right = Some(branch.hash.clone().unwrap());
                    }
                    branch.hash_and_store(db);

                    //new_root = root.clone();
                }
                _ => {
                    panic!("This should never happen!")
                }
            }
        }
    }
    new_root
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

#[cfg(test)]
mod tests {
    use crate::merkle::tests::{generate_random_data, generate_random_key};
    #[cfg(feature = "sqlite")]
    use crate::store::db::sql::TrieDB;
    #[cfg(not(feature = "sqlite"))]
    use crate::store::db::TrieDB;
    use crate::store::types::{Hashable, Node, Root};
    use crate::store::{db::Database, types::Leaf};
    use crate::{check_leaf, insert_leaf, update_leaf};
    use colored::*;
    use indicatif::ProgressBar;
    use std::collections::HashMap;
    use std::time::Instant;
    #[test]
    fn test_insert_leaf() {
        let start_time = Instant::now();
        #[cfg(not(feature = "sqlite"))]
        let mut db = TrieDB {
            nodes: HashMap::new(),
        };

        #[cfg(feature = "sqlite")]
        let mut db = TrieDB {
            path: env::var("PATH_TO_DB").unwrap_or("database.sqlite".to_string()),
            cache: None,
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
        let new_root = insert_leaf(&mut db, &mut leaf_1, root_node.clone());
        let _ = insert_leaf(&mut db, &mut leaf_2, Node::Root(new_root));

        println!(
            "{} Elapsed Time: {} µs",
            "[1x Insert]".yellow(),
            &start_time.elapsed().as_micros().to_string().blue()
        );
    }
    #[test]
    /*fn test_update_leaf() {
        let start_time = Instant::now();
        #[cfg(not(feature = "sqlite"))]
        let mut db = TrieDB {
            nodes: HashMap::new(),
        };

        #[cfg(feature = "sqlite")]
        let mut db = TrieDB {
            path: env::var("PATH_TO_DB").unwrap_or("database.sqlite".to_string()),
            cache: None,
        };
        let mut leaf_1: Leaf = Leaf::empty(vec![0u8; 256]);
        leaf_1.hash();
        let root: Root = Root::empty();
        let root_node = Node::Root(root);
        let new_root = insert_leaf(&mut db, &mut leaf_1, root_node);
        let mut leaf_1_updated: Leaf = Leaf::empty(vec![0; 256]);
        leaf_1_updated.data = Some(vec![1]);
        let _new_root = update_leaf(&mut db, &mut leaf_1_updated, Node::Root(new_root));
        let _leaf_from_db = db.get(&leaf_1_updated.hash.unwrap()).unwrap();
        println!(
            "{} Elapsed Time: {} µs",
            "[1x Update]".yellow(),
            &start_time.elapsed().as_micros().to_string().blue()
        );
    }*/
    #[test]
    fn test_many_leafs() {
        let transaction_count: u32 = std::env::var("INSERT_TRANSACTION_COUNT")
            .unwrap_or_else(|_| "4".to_string())
            .parse::<u32>()
            .expect("Invalid argument STRESS_TEST_TRANSACTION_COUNT");
        let mut transactions: Vec<Leaf> = Vec::new();
        for _ in 0..transaction_count {
            let leaf_key = generate_random_key();
            let leaf: Leaf = Leaf::new(leaf_key, Some(generate_random_data())); //Some(generate_random_data()));
            transactions.push(leaf);
        }
        let start_time = Instant::now();
        #[cfg(not(feature = "sqlite"))]
        let mut db = TrieDB {
            nodes: HashMap::new(),
        };

        #[cfg(feature = "sqlite")]
        let mut db = TrieDB {
            path: env::var("PATH_TO_DB").unwrap_or("database.sqlite".to_string()),
            cache: None,
        };
        let root: Root = Root::empty();
        let mut root_node = Node::Root(root);
        let progress_bar: ProgressBar = ProgressBar::new(transaction_count as u64);
        for mut leaf in transactions {
            leaf.hash();
            let new_root = insert_leaf(&mut db, &mut leaf, root_node.clone());
            /*assert!(check_leaf(
                &mut db,
                leaf.clone(),
                Node::Root(new_root.clone())
            ));*/
            root_node = Node::Root(new_root.clone());
            progress_bar.inc(1);
        }
        progress_bar.finish_with_message("Done testing insert!");
        println!(
            "[{}x Insert] Elapsed Time: {} s",
            transaction_count.to_string().yellow(),
            &start_time.elapsed().as_secs().to_string().blue()
        );
        #[cfg(not(feature = "sqlite"))]
        println!("Memory DB size: {}", &db.nodes.len().to_string().blue());
    }

    #[cfg(feature = "sqlite")]
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
        let new_root = insert_leaf(&mut db, &mut leaf_2, Node::Root(new_root));
        assert_eq!(
            new_root.hash.clone().unwrap(),
            Root {
                hash: Some(vec![
                    170, 229, 131, 77, 235, 12, 173, 127, 222, 26, 105, 40, 22, 13, 179, 45, 178,
                    246, 170, 244, 16, 171, 204, 67, 102, 94, 208, 139, 143, 112, 136, 169
                ]),
                left: Some(vec![
                    192, 255, 218, 137, 120, 169, 46, 169, 51, 142, 15, 1, 84, 251, 124, 134, 95,
                    25, 100, 240, 136, 56, 116, 145, 21, 237, 3, 48, 55, 36, 46, 197
                ]),
                right: None
            }
            .hash
            .unwrap()
        );
        println!(
            "{} Elapsed Time: {} µs",
            "[1x Insert]".yellow(),
            &start_time.elapsed().as_micros().to_string().blue()
        );
        let mut leaf_1: Leaf = Leaf::empty(vec![1u8; 256]);
        leaf_1.hash();
        let root: Root = new_root;
        let root_node: Node = Node::Root(root);
        let new_root: Root = insert_leaf(&mut db, &mut leaf_1, root_node);
        let proof = merkle_proof(&mut db, leaf_1.key, Node::Root(new_root.clone()));
        let inner_proof = proof.unwrap().nodes;
        verify_merkle_proof(inner_proof, new_root.hash.clone().unwrap());
    }
}
