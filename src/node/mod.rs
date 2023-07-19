mod ext;
mod internal;
mod leaf;
mod suffix;

pub use ext::ExtensionNode;
pub use internal::InternalNode;
pub use leaf::LeafNode;
pub use suffix::SuffixTree;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::Committer;

pub enum Node<C: Committer> {
    Empty,
    Leaf(Rc<RefCell<LeafNode<C>>>),
    Suffix(Rc<RefCell<SuffixTree<C>>>),
    Stem(Rc<RefCell<InnerNode<C>>>),
}

pub enum InnerNode<C: Committer> {
    Extension(Rc<RefCell<ExtensionNode<C>>>),
    Internal(Rc<RefCell<InternalNode<C>>>),
}

impl<C: Committer> InnerNode<C> {
    pub fn get_c_field(&self) -> &C::Scalar {
        match self {
            InnerNode::Extension(e) => e.borrow().get_c_field(),
            InnerNode::Internal(i) => i.borrow().get_c_field(),
        }
    }
}

pub trait CommitNode<C: Committer> {
    fn compute_c(&mut self, committer: &C);

    fn get_c(&self) -> &C::Point;

    fn get_c_field(&self) -> &C::Scalar;
}
