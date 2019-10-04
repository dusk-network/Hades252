use crate::errors::PermError;
use crate::linear_combination::{ConstraintSystem, LinearCombination};
use crate::mds_matrix::MDS_MATRIX;
use crate::scalar::Scalar;

pub fn hash(
    cs: &mut dyn ConstraintSystem,
    data: &[LinearCombination],
) -> Result<LinearCombination, PermError> {
    let width = MDS_MATRIX.len();

    if data.len() >= width - 1 {
        return Err(PermError::InputFull);
    }

    // The base type declares the output type, so we use u64
    // since the arity of the merkle tree is not likely to be
    // >= 2^64.
    let first_item = LinearCombination::from(Scalar::from((1u64 << width) - 1));

    let mut words = vec![LinearCombination::from(Scalar::zero()); width];
    let words_slice = &mut words[1..1 + data.len()];

    words_slice.clone_from_slice(data);
    words[0] = first_item;

    let words = super::perm(cs, words)?;
    Ok(words[1].clone())
}
