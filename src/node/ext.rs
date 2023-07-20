use crate::{Committer, KeyStem};

use super::{CommitNode, StemNode, SuffixTree};

pub struct ExtensionNode<C: Committer> {
    pub stem: KeyStem,
    stem_field: C::Scalar,
    one_field: C::Scalar,
    c: C::Point, //C = Commit(1, stem, C1, C2)
    pub c_field: C::Scalar,
    suffix_tree: SuffixTree<C>,
}

impl<C: Committer> StemNode<C> for ExtensionNode<C> {
    fn get_c_field(&self) -> &<C as Committer>::Scalar {
        &self.c_field
    }
}

impl<C: Committer> CommitNode<C> for ExtensionNode<C> {
    fn compute_c(&mut self, committer: &C) -> () {
        self.c = committer.commit(vec![
            &self.one_field,
            &self.stem_field,
            &self.suffix_tree.get_c1_field(),
            &self.suffix_tree.get_c2_field(),
        ]);

        self.c_field = C::from_point_to_scalar(&self.c);
    }

    fn get_c(&self) -> &<C as Committer>::Point {
        &self.c
    }

    fn get_c_field(&self) -> &<C as Committer>::Scalar {
        &self.c_field
    }
}

impl<C: Committer> ExtensionNode<C> {
    fn get_suffix_tree(&self) -> &SuffixTree<C> {
        &self.suffix_tree
    }
}
