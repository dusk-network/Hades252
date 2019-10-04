use crate::errors::PermError;
use crate::linear_combination::{ConstraintSystem, LinearCombination};

pub fn hash(
    _cs: &mut dyn ConstraintSystem,
    _data: &[LinearCombination],
) -> Result<LinearCombination, PermError> {
    unimplemented!()
}
