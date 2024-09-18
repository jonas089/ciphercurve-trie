use std::io::{Error, ErrorKind};
pub enum TrieError {
    DuplicateLeaf,
    InvalidChild,
    InvalidParent,
    InvalidBranch,
    MissingNode,
}

impl From<TrieError> for Error {
    fn from(e: TrieError) -> Self {
        match e {
            TrieError::DuplicateLeaf => Error::new(ErrorKind::Other, "DuplicateLeaf"),
            TrieError::InvalidChild => Error::new(ErrorKind::Other, "InvalidChild"),
            TrieError::InvalidParent => Error::new(ErrorKind::Other, "InvalidParent"),
            TrieError::InvalidBranch => Error::new(ErrorKind::Other, "InvalidBranch"),
            TrieError::MissingNode => Error::new(ErrorKind::Other, "MissingNode"),
        }
    }
}
