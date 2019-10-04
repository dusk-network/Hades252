use crate::errors::PermError;
use bulletproofs::r1cs::{ConstraintSystem, LinearCombination};

pub fn hash(
    _cs: &mut dyn ConstraintSystem,
    _data: &[LinearCombination],
) -> Result<LinearCombination, PermError> {
    unimplemented!()
}
