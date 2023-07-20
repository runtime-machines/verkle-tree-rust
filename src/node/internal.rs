use crate::{Committer, BRANCHING_FACTOR_WIDTH};

use super::{CommitNode, StemNode};

pub struct InternalNodeChildren<C: Committer, S: StemNode<C>>(Vec<S>);

impl<C: Committer, S: StemNode<C>> InternalNodeChildren<C, S> {
    pub fn add(&mut self, child: dyn StemNode<C>) {
        if self.0.len() >= BRANCHING_FACTOR_WIDTH {
            panic!()
        } else {
            self.0.push(child);
        }
    }

    pub fn as_scalars(&self) -> Vec<&C::Scalar> {
        self.0.iter().map(|n| n.get_c_field()).collect()
    }
}

pub struct InternalNode<C: Committer, S: StemNode<C>> {
    pub index: u8, // 1 byte from 0 to BRANCHING_FACTOR_WIDTH-1
    // if the stems (of two different values) differ at the third byte,
    // two internal nodes are added until the differing byte
    pub c: C::Point,
    pub c_field: C::Scalar,
    pub children: InternalNodeChildren<C, S>,
}

impl<C: Committer, S: StemNode<C>> StemNode<C> for InternalNode<C, S> {
    fn get_c_field(&self) -> &<C as Committer>::Scalar {
        &self.c_field
    }
}

impl<C: Committer, S: StemNode<C>> CommitNode<C> for InternalNode<C, S> {
    fn compute_c(&mut self, committer: &C) {
        self.c = committer.commit(self.children.as_scalars());
    }

    fn get_c(&self) -> &<C as Committer>::Point {
        &self.c
    }

    fn get_c_field(&self) -> &<C as Committer>::Scalar {
        &self.c_field
    }
}
