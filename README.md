⚠️ This is not a production library ⚠️
# Merkle Trie for Blockchain Systems
This Trie can be used to represent state in Blockchain systems.

## Todos
- Hashing-agnostic design would be preferred. Hasher Impl will benefit the design.
- ZK-friendly optimization features, starting with a Risc0 Hashing optimization feature.
- Collision handler
- Write comprehensive tests
- Benchmark time and space complexity

## Todos: Optimization
- Decrease depth by merging Nodes
- Store leafs in the DB with Key and store a Reference to these leafs in the populated Branches
