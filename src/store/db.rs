use crate::store::types::Node;
use std::collections::HashMap;

pub trait Database {
    fn insert(&mut self, key: &[u8], node: Node);
    fn get(&mut self, key: &[u8]) -> Option<&mut Node>;
}

#[derive(Debug)]
pub struct InMemoryDB {
    pub nodes: HashMap<Vec<u8>, Node>,
}
impl Database for InMemoryDB {
    fn insert(&mut self, key: &[u8], node: Node) {
        self.nodes.insert(key.to_vec(), node);
    }
    fn get(&mut self, key: &[u8]) -> Option<&mut Node> {
        self.nodes.get_mut(key)
    }
}

//#[cfg(feature = "sqlite")]
pub mod sql {
    extern crate rusqlite;
    use super::Database;
    use crate::store::types::Node;
    use rusqlite::{params, Connection};

    pub struct SqLiteDB {
        pub path: &'static str,
        pub cache: Option<Node>,
    }
    impl SqLiteDB {
        pub fn setup(&self) {
            let conn = Connection::open(self.path).unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS nodes (
                          key    BLOB PRIMARY KEY,
                          node   BLOB NOT NULL
                          )",
                [],
            )
            .unwrap();
        }
    }
    impl Database for SqLiteDB {
        fn insert(&mut self, key: &[u8], node: Node) {
            let conn = Connection::open(self.path).unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO nodes (key, node) VALUES (?1, ?2)",
                params![key, bincode::serialize(&node).unwrap()],
            )
            .unwrap();
        }
        fn get(&mut self, key: &[u8]) -> Option<&mut Node> {
            let conn = Connection::open(self.path).unwrap();
            let mut stmt = conn
                .prepare("SELECT node FROM nodes WHERE key = ?1 LIMIT 1")
                .unwrap();

            let node_serialized: Option<Vec<u8>> = stmt
                .query_row([&key], |row| {
                    let node_serialized: Vec<u8> = row.get(0).unwrap();
                    Ok(Some(node_serialized))
                })
                .unwrap_or(None);

            if let Some(node_serialized) = node_serialized {
                let node: Node = bincode::deserialize(&node_serialized).unwrap();
                self.cache = Some(node);
                self.cache.as_mut()
            } else {
                None
            }
        }
    }
}
