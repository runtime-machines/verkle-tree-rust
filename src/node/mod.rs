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

use crate::Committer;

pub enum Node<C: Committer, S: StemNode<C>> {
    Empty,
    Leaf(Rc<RefCell<LeafNode<C>>>),
    Extension(Rc<RefCell<ExtensionNode<C>>>),
    Internal(Rc<RefCell<InternalNode<C, S>>>),
}

pub trait StemNode<C: Committer> {
    fn get_c_field(&self) -> &C::Scalar;
}

pub trait CommitNode<C: Committer> {
    fn compute_c(&mut self, committer: &C);

    fn get_c(&self) -> &C::Point;

    fn get_c_field(&self) -> &C::Scalar;
}
