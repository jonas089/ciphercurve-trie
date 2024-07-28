// Compute Merkle Proof for a Leaf at a given point in time (e.g. at a Snapshot)
use crate::store::db::InMemoryDB;
// obtain the merkle path for a leaf
pub fn merkle_proof(db: &mut InMemoryDB, key: Vec<u8>) {
    assert_eq!(key.len(), 256);
}
