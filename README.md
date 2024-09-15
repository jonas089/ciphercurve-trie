# Merkle Binary Patricia Trie for immutable Blockchain State
![benchmark](https://github.com/jonas089/jonas089-trie/blob/master/resources/simple-bench.png)

## Implementation Details

Historical state is preserved for each `root hash`, one can query the `db` for a Root and generate `Merkle Proofs` for `Leaf`s in the `Trie`.
Each `Merkle Proof` is verified against a `root` to verify that a `Leaf` was present in the `Trie` at some point in time.

An example of constructing the in-memory `db`, inserting a `Leafs` and verifying `Merkle Proof` can be found [here](https://github.com/jonas089/jonas089-trie/blob/master/src/merkle.rs)

## Experimental SQLite Support
New Feature! The InMemoryDB is now optional and can be replaced with an SQLite table.

To enable this feature, include the `sqlite` flag:

```rust
cargo test --features sqlite test_sql_db
```


## API

This library primarily exposes two entry points, one to insert a new `Leaf` into a `Trie`:

```rust
pub fn insert_leaf(db: &mut dyn Database, new_leaf: &mut Leaf, root_node: Node) -> Root {
    assert_eq!(new_leaf.key.len(), 256);
    let modified_nodes = traverse_trie(db, new_leaf, root_node.clone(), false);
    let mut new_root = update_modified_leafs(db, modified_nodes, root_node.unwrap_as_root());
    new_root.hash_and_store(db);
    new_root
}
```

pub fn update_leaf(db: &mut InMemoryDB, new_leaf: &mut Leaf, root_node: Node) -> Root {
    let (modified_nodes, new_root) = traverse_trie(db, new_leaf, root_node, true);
    let mut new_root = update_modified_leafs(db, modified_nodes, new_root);
    new_root.hash_and_store(db);
    new_root
}
```

Additionally, there are two public functions to generate and verify `Merkle Proofs`:

```rust
pub fn merkle_proof(db: &mut dyn Database, key: Vec<u8>, trie_root: Node) -> Option<MerkleProof> {
    assert_eq!(key.len(), 256);
    let mut proof: MerkleProof = MerkleProof { nodes: Vec::new() };
    let mut current_node = trie_root.clone();
    loop {
        match &mut current_node {
            Node::Root(root) => {
                proof.nodes.push((false, Node::Root(root.clone())));
                if key[0] == 0 {
                    let left_child = db.get(&root.left.clone().unwrap()).unwrap();
                    current_node = left_child.clone();
                    proof.nodes.push((false, left_child.clone()));
                } else {
                    let right_child = db.get(&root.right.clone().unwrap()).unwrap();
                    current_node = right_child.clone();
                    proof.nodes.push((true, right_child.clone()));
                }
            }
            Node::Branch(branch) => {
                let digit = key[branch.key[0] as usize];
                if digit == 0 {
                    current_node = db.get(&branch.left.clone().unwrap()).unwrap().clone();
                    proof.nodes.push((false, current_node.clone()));
                } else {
                    current_node = db.get(&branch.right.clone().unwrap()).unwrap().clone();
                    proof.nodes.push((true, current_node.clone()));
                }
            }
            Node::Leaf(_) => return Some(proof),
        }
    }
}

```
