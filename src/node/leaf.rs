use crate::{
    Committer, KeySuffix, LeafNodeValue, LEAF_HALF_VALUE_SIZE, LEAF_VALUE_SIZE,
};

pub struct UpperValue([u8; LEAF_HALF_VALUE_SIZE]);
pub struct LowerValue([u8; LEAF_HALF_VALUE_SIZE + 1]);

//total size 33bytes ( 32 bytes + 1 bit)
pub struct LeafNode<C: Committer> {
    pub suffix: KeySuffix, //for left from 0 to 127, for right from 128 to 255
    pub value: LeafNodeValue,
    pub lower: LowerValue,
    pub lower_field: C::Scalar,
    pub upper: UpperValue,
    pub upper_field: C::Scalar,
    pub modifier: bool, //this could be removed
                        /*
                            // lower++upper (without considering modifier) = leaf_value
                            pub lower: LowerValue, // + 1 bit (129th bit) to differentiate between a leaf that
                            // has never been accessed and a leaf that has been overwritten with 0s
                            // 1=accessed 0=never accessed
                            pub upper: UpperValue,

                        */
}

impl<C: Committer> LeafNode<C> {
    pub fn new(suffix: KeySuffix, value: LeafNodeValue) -> Self {
        let modifier = false;
        let (lower, upper) = Self::split_node_value(value, modifier);
        let lower_field = C::from_bytes_to_scalar(lower.0.as_slice());
        let upper_field = C::from_bytes_to_scalar(upper.0.as_slice());

        LeafNode {
            suffix,
            value,
            modifier,
            lower,
            lower_field,
            upper,
            upper_field,
        }
    }

    pub fn update(mut self, value: LeafNodeValue) -> () {
        self.modifier = true;
        self.value = value;

        let (lower, upper) = Self::split_node_value(self.value, self.modifier);

        self.lower = lower;
        self.upper = upper;
    }

    fn split_node_value(
        value: LeafNodeValue,
        modifier: bool,
    ) -> (LowerValue, UpperValue) {
        let mut lower: LowerValue = LowerValue(Default::default());
        lower.0[0..LEAF_HALF_VALUE_SIZE + 1]
            .copy_from_slice(&value[0..LEAF_HALF_VALUE_SIZE + 1]);
        lower.0[LEAF_HALF_VALUE_SIZE] = {
            if modifier {
                1
            } else {
                0
            }
        };

        let upper: UpperValue = UpperValue(
            value[LEAF_HALF_VALUE_SIZE..LEAF_VALUE_SIZE]
                .try_into()
                .expect("error encoding value"),
        );

        (lower, upper)
    }

    pub fn get_value_as_scalars(&self) -> Vec<&<C as Committer>::Scalar> {
        vec![&self.lower_field, &self.upper_field]
    }
}
