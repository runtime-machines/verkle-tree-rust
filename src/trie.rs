use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::{Hasher, Node, TrieError, DB};

pub type TrieResult<T> = Result<T, TrieError>;

pub trait Trie<D: DB, H: Hasher> {
    /// Returns the value for key stored in the trie.
    fn get(&self, key: &[u8]) -> TrieResult<Option<Vec<u8>>>;

    /// Checks that the key is present in the trie
    fn contains(&self, key: &[u8]) -> TrieResult<bool>;

    /// Inserts value into trie and modifies it if it exists
    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) -> TrieResult<()>;

    /// Removes any existing value for key from the trie.
    fn remove(&mut self, key: &[u8]) -> TrieResult<bool>;

    /// Saves all the nodes in the db, clears the cache data, recalculates the root.
    /// Returns the root hash of the trie.
    fn root(&mut self) -> TrieResult<Vec<u8>>;

    /// Prove constructs a merkle proof for key. The result contains all encoded nodes
    /// on the path to the value at key. The value itself is also included in the last
    /// node and can be retrieved by verifying the proof.
    ///
    /// If the trie does not contain a value for key, the returned proof contains all
    /// nodes of the longest existing prefix of the key (at least the root node), ending
    /// with the node that proves the absence of the key.
    fn get_proof(&self, key: &[u8]) -> TrieResult<Vec<Vec<u8>>>;

    /// return value if key exists, None if key not exist, Error if proof is wrong
    fn verify_proof(
        &self,
        root_hash: &[u8],
        key: &[u8],
        proof: Vec<Vec<u8>>,
    ) -> TrieResult<Option<Vec<u8>>>;
}

pub struct VerkleTrie<D: DB, H: Hasher> {
    root: Node,
    root_hash: Vec<u8>,

    db: Arc<D>,
    hasher: Arc<H>,
    backup_db: Option<Arc<D>>,

    cache: RefCell<HashMap<Vec<u8>, Vec<u8>>>,
    passing_keys: RefCell<HashSet<Vec<u8>>>,
    gen_keys: RefCell<HashSet<Vec<u8>>>,
}
