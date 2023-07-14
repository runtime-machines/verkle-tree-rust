use std::cell::RefCell;
use std::rc::Rc;

const LEAF_VALUE_SIZE: usize = 32; // 256 bits
const KEY_BYTES: usize = 32;
const BRANCHING_FACTOR_WIDTH: usize = 256; // if more then 256 remember to check and change if needed u8 to u32
                                           // for InternalNode::index (only if 256+), LeafNode::suffix (only if 1024+)

//key consists of a stem of 31 bytes and a suffix of one byte for a total of 32 bytes

pub struct Key {
    pub stem: [u8; KEY_BYTES - 1], //31
    pub suffix: u8,                //1
}

pub enum InnerNode {
    Extension(Rc<RefCell<ExtensionNode>>),
    Internal(Rc<RefCell<InternalNode>>),
}

pub enum Node {
    Empty,
    Leaf(Rc<RefCell<LeafNode>>),
    Suffix(Rc<RefCell<SuffixTree>>),
    Stem(Rc<RefCell<InnerNode>>),
}

//16 bytes + 1 bit
pub struct LeafNodeLowerValue {
    pub value: [u8; LEAF_VALUE_SIZE / 2],
    pub modified: u8, // + 1 bit (129th bit) to differentiate between a leaf that
                      // has never been accessed and a leaf that has been overwritten with 0s
                      // 1=accessed 0=never accessed
}

//16 bytes
pub struct LeafNodeUpperValue {
    pub value: [u8; LEAF_VALUE_SIZE / 2],
}

pub struct LeafNodeValue {
    // lower++upper (without considering modifier) = leaf_value
    pub lower: LeafNodeLowerValue, //this is also called field element
    pub upper: LeafNodeUpperValue, //this is also called filed element
}

pub struct LeafNode {
    pub suffix: u8, //for left from 0 to 127, for right from 128 to 255
    //total size 33bytes ( 32 bytes + 1 bit)
    pub value: LeafNodeValue,
}

pub struct SuffixTree {
    // C1 = Commit(v0_lower_modifier, v0_upper, ..., v127_lower_modifier, v127_upper)
    pub left: [LeafNode; BRANCHING_FACTOR_WIDTH / 2],
    // C2 = Commit(v128_lower_modifier, v128_upper, ..., v255_lower_modifier, v255_upper)
    pub right: [LeafNode; BRANCHING_FACTOR_WIDTH / 2],
}

pub struct ExtensionNode {
    //C = Commit(1, stem, C1, C2)
    pub stem: [u8; KEY_BYTES - 1], // full stem
    pub suffix_tree: SuffixTree,
}

pub struct InternalNode {
    pub index: u8, // 1 byte from 0 to BRANCHING_FACTOR_WIDTH-1
    // if the stems (of two different values) differ at the third byte,
    // two internal nodes are added until the differing byte
    pub childs: [InnerNode; BRANCHING_FACTOR_WIDTH],
}
