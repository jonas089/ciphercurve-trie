⚠️ This is not a production library ⚠️

⚠️ Optimization work in progress ⚠️

Refactor is in progress, see [here](https://github.com/jonas089/jonas089-trie/tree/refactor). I am making good progress and the new Trie actually maintains state (the one in the master branch won't work for actual blockchain use).

Once the refactoring on the `refactor` branch is complete, `master` will be overriden entirely. Using the Trie on the `master` branch for a blockchain system is not recommended.

*This codebase will be refactored, see Todos: Optimization*
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
- Store modified leafs and branches in memory and re-hash them once insert concluded
