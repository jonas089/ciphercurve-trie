# Merkle Binary Patricia Trie for immutable Blockchain State

:warning: This Library has not been audited, use at own Risk! :warning:

![benchmark](https://github.com/jonas089/jonas089-trie/blob/master/resources/simple-bench.png)

## Implementation Details

Historical state is preserved for each `root hash`, one can query the `db` for a Root and generate `Merkle Proofs` for `Leaf`s in the `Trie`.
Each `Merkle Proof` is verified against a `root` to verify that a `Leaf` was present in the `Trie` at some point in time.

An example of constructing the in-memory `db`, inserting a `Leafs` and verifying `Merkle Proof` can be found [here](https://github.com/jonas089/jonas089-trie/blob/master/src/merkle.rs)

## API

This library primarily exposes two entry points, one to insert a new `Leaf` into a `Trie` and one to update an existing `Leaf` in the `Trie`:

```rust
pub fn insert_leaf(db: &mut InMemoryDB, new_leaf: &mut Leaf, root_node: Node) -> Root {
    assert_eq!(new_leaf.key.len(), 256);
    let (modified_nodes, new_root) = traverse_trie(db, new_leaf, root_node, false);
    let mut new_root = update_modified_leafs(db, modified_nodes, new_root);
    new_root.hash_and_store(db);
    new_root
}

pub fn update_leaf(db: &mut InMemoryDB, new_leaf: &mut Leaf, root_node: Node) -> Root {
    let (modified_nodes, new_root) = traverse_trie(db, new_leaf, root_node, true);
    let mut new_root = update_modified_leafs(db, modified_nodes, new_root);
    new_root.hash_and_store(db);
    new_root
}
```
