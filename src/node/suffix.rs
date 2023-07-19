use crate::{Committer, BRANCHING_FACTOR_WIDTH};

use super::LeafNode;

pub struct SuffixTreeBranch<C: Committer>(Vec<LeafNode<C>>);

impl<C: Committer> SuffixTreeBranch<C> {
    pub fn add(&mut self, child: LeafNode<C>) {
        if self.0.len() >= BRANCHING_FACTOR_WIDTH / 2 {
            panic!()
        } else {
            self.0.push(child);
        }
    }
}

pub struct SuffixTree<C: Committer> {
    c1: C::Point, // C1 = Commit(v0_lower_modifier, v0_upper, ..., v127_lower_modifier, v127_upper)
    c1_field: C::Scalar,
    left_branch: SuffixTreeBranch<C>,
    c2: C::Point, // C2 = Commit(v128_lower_modifier, v128_upper, ..., v255_lower_modifier, v255_upper)
    c2_field: C::Scalar,
    right_branch: SuffixTreeBranch<C>,
}

impl<C: Committer> SuffixTree<C> {
    pub fn compute_c(&mut self, committer: &C) {
        let compute_branch_commit = |b: &Vec<LeafNode<C>>| {
            committer.commit(
                b.iter()
                    .map(|l| l.get_value_as_scalars())
                    .flatten()
                    .collect(),
            )
        };

        //this two operation can be done concurrently
        self.c1 = compute_branch_commit(&self.left_branch.0); //self.compute_commit(&self.left_branch, committer);
        self.c2 = compute_branch_commit(&self.right_branch.0); //self.compute_commit(&self.right_branch, committer);

        self.c1_field = C::from_point_to_scalar(&self.c1);
        self.c2_field = C::from_point_to_scalar(&self.c2);
    }

    pub fn get_c2_field(&self) -> &C::Scalar {
        &self.c2_field
    }

    pub fn get_c1_field(&self) -> &C::Scalar {
        &self.c1_field
    }

    pub fn get_c2(&self) -> &C::Point {
        &self.c2
    }

    pub fn get_c1(&self) -> &C::Point {
        &self.c1
    }

    pub fn get_left_branch(&self) -> &SuffixTreeBranch<C> {
        &self.left_branch
    }

    pub fn get_right_branch(&self) -> &SuffixTreeBranch<C> {
        &self.right_branch
    }
}
