use crate::errors::PermError;
use crate::MERKLE_WIDTH;
use curve25519_dalek::scalar::Scalar;

pub fn hash(_data: &[Scalar]) -> Result<Scalar, PermError> {
    let mut leaves = [Scalar::zero(); MERKLE_WIDTH];
    unimplemented!()
}
