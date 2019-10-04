use crate::errors::PermError;
use crate::mds_matrix::MDS_MATRIX;
use curve25519_dalek::scalar::Scalar;

pub fn hash(data: &[Scalar]) -> Result<Scalar, PermError> {
    let width = MDS_MATRIX.len();

    if data.len() >= width - 1 {
        return Err(PermError::InputFull);
    }

    // The base type declares the output type, so we use u64
    // since the arity of the merkle tree is not likely to be
    // >= 2^64.
    let first_item = Scalar::from((1u64 << width) - 1);

    let mut words = vec![Scalar::zero(); width];
    let words_slice = &mut words[1..1 + data.len()];

    words_slice.copy_from_slice(data);
    words[0] = first_item;

    let words = super::perm(words)?;
    Ok(words[1])
}
