mod committer;
mod db;
mod errors;
mod hasher;
mod node;
mod trie;

pub use committer::Committer;
pub use db::DB;
pub use errors::TrieError;
pub use hasher::Hasher;
pub use node::Node;
use node::{InnerNode, LeafNode};
pub use trie::{TrieResult, VerkleTrie};

const LEAF_VALUE_SIZE: usize = 32; // 256 bits //should be always mutiple of 2 and less of 512 bits
const LEAF_HALF_VALUE_SIZE: usize = LEAF_VALUE_SIZE / 2;
const KEY_BYTES: usize = 32; //
const BRANCHING_FACTOR_WIDTH: usize = 256; // if more then 256 remember to check and change if needed u8 to u32
                                           // for InternalNode::index (only if 256+), LeafNode::suffix (only if 1024+)
const KEY_SUFFIX_BYTES: usize = 1;

//key consists of a stem of 31 bytes and a suffix of one byte for a total of 32 bytes
pub type Key = [u8; KEY_BYTES];
pub type KeyStem = [u8; KEY_BYTES - KEY_SUFFIX_BYTES];
pub type KeySuffix = [u8; KEY_SUFFIX_BYTES];
pub type SuffixTreeBranch = Vec<LeafNode>; //; BRANCHING_FACTOR_WIDTH / 2];
pub type InnerNodeChildren<C> = Vec<InnerNode<C>>; // BRANCHING_FACTOR_WIDTH];
pub type LeafNodeValue = [u8; LEAF_VALUE_SIZE];
pub type UpperValue = [u8; LEAF_HALF_VALUE_SIZE];
pub type LowerValue = [u8; LEAF_HALF_VALUE_SIZE + 1];
