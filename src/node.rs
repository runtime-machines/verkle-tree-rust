use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    Committer, InnerNodeChildren, KeyStem, KeySuffix, LeafNodeValue,
    LowerValue, SuffixTreeBranch, UpperValue, LEAF_HALF_VALUE_SIZE,
    LEAF_VALUE_SIZE,
};

pub enum InnerNode<C: Committer> {
    Extension(Rc<RefCell<ExtensionNode<C>>>),
    Internal(Rc<RefCell<InternalNode<C>>>),
}

pub enum Node<C: Committer> {
    Empty,
    Leaf(Rc<RefCell<LeafNode>>),
    Suffix(Rc<RefCell<SuffixTree<C>>>),
    Stem(Rc<RefCell<InnerNode<C>>>),
}

//total size 33bytes ( 32 bytes + 1 bit)
pub struct LeafNode {
    pub suffix: KeySuffix, //for left from 0 to 127, for right from 128 to 255

    pub value: LeafNodeValue,
    pub modifier: bool,
    /*
        // lower++upper (without considering modifier) = leaf_value
        pub lower: LowerValue, // + 1 bit (129th bit) to differentiate between a leaf that
        // has never been accessed and a leaf that has been overwritten with 0s
        // 1=accessed 0=never accessed
        pub upper: UpperValue,

    */
}

impl LeafNode {
    pub fn new(suffix: KeySuffix, value: LeafNodeValue) -> Self {
        LeafNode {
            suffix,
            value,
            modifier: false,
        }
    }

    pub fn update(mut self, value: LeafNodeValue) -> () {
        self.modifier = true;
        self.value = value;
    }

    //TODO: check if its better to pre calulate it or
    //to call it when needed
    fn split_value(&self) -> Vec<&[u8]> {
        let mut lower: LowerValue = Default::default();
        lower[0..LEAF_HALF_VALUE_SIZE + 1]
            .copy_from_slice(&self.value[0..LEAF_HALF_VALUE_SIZE + 1]);
        lower[LEAF_HALF_VALUE_SIZE] = {
            if self.modifier {
                1
            } else {
                0
            }
        };

        let upper: UpperValue = self.value
            [LEAF_HALF_VALUE_SIZE..LEAF_VALUE_SIZE]
            .try_into()
            .expect("Node value has not 32 bytes");

        vec![lower.as_slice(), upper.as_slice()]
    }
}

pub struct SuffixTree<C: Committer> {
    // C1 = Commit(v0_lower_modifier, v0_upper, ..., v127_lower_modifier, v127_upper)
    pub c1: C::Commit,
    pub left_branch: SuffixTreeBranch,
    // C2 = Commit(v128_lower_modifier, v128_upper, ..., v255_lower_modifier, v255_upper)
    pub c2: C::Commit,
    pub right_branch: SuffixTreeBranch,
}

impl<C: Committer> SuffixTree<C> {
    pub fn compute_c_1(mut self, committer: C) {
        self.c1 = self.compute_commit(&self.left_branch, committer);
    }

    pub fn compute_c_2(mut self, committer: C) {
        self.c2 = self.compute_commit(&self.right_branch, committer);
    }

    fn compute_commit(
        &self,
        branch: &SuffixTreeBranch,
        committer: C,
    ) -> C::Commit {
        committer.commit(Self::pack_leaves(branch)).consume_commit()
    }

    pub fn pack_leaves(branch: &SuffixTreeBranch) -> Vec<&[u8]> {
        branch.iter().map(|n| n.split_value()).flatten().collect()
    }
}
pub struct ExtensionNode<C: Committer> {
    pub stem: KeyStem,
    //C = Commit(1, stem, C1, C2)
    pub c: C::Commit,
    pub suffix_tree: SuffixTree<C>,
}

impl<C: Committer> ExtensionNode<C> {
    pub fn compute_c(&self, committer: C) -> () {
        let m_stem = committer.from_bytes_to_scalar(&self.stem);
        let one = committer.from_bytes_to_scalar(&[1]);

        committer.commit(vec![
            one,
            m_stem,
            &self.suffix_tree.c1,
            &self.suffix_tree.c2,
        ]);
    }
}

pub struct InternalNode<C: Committer> {
    pub index: u8, // 1 byte from 0 to BRANCHING_FACTOR_WIDTH-1
    // if the stems (of two different values) differ at the third byte,
    // two internal nodes are added until the differing byte
    pub c: C,
    pub children: InnerNodeChildren<C>,
}
