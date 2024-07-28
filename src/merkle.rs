// Compute Merkle Proof for a Leaf at a given point in time (e.g. at a Snapshot)
use crate::store::{db::InMemoryDB, types::Node};
// obtain the merkle path for a leaf
pub fn merkle_proof(db: &mut InMemoryDB, key: Vec<u8>) {
    assert_eq!(key.len(), 256);
}

/* Todo
Construct merkle path for a node given the path

Root
-> left (0)
-> right (1)

Branch
-> prefix[] or [...]
if []: current_idx 0: left, current_idx 1: right
if [...] current_idx [...] 0: left, current_idx [...] 1: right

Leaf
-> ok

collect all siblings in a list and re-hash up to the root to verify.
*/

struct MerkleProof {
    nodes: Vec<(bool, Node)>,
}
