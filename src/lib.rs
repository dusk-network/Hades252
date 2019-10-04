#![feature(test)]

// Poseidon constants
pub(crate) const WIDTH: usize = 9;
pub(crate) const FULL_ROUNDS: usize = 8;
pub(crate) const PARTIAL_ROUNDS: usize = 59;

// Merkle constants
pub(crate) const MERKLE_ARITY: usize = 8;
pub(crate) const MERKLE_HEIGHT: usize = 3;
pub(crate) const MERKLE_WIDTH: usize = 64;

pub mod errors;
pub mod linear_combination;
mod mds_matrix;
mod round_constants;
pub mod scalar;

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn constants_consistency() {
        // Grant we have enough constants for the rounds setup
        assert!(WIDTH * (FULL_ROUNDS + PARTIAL_ROUNDS) <= round_constants::ROUND_CONSTANTS.len());

        // Enforce a relation between the provided MDS matrix and the arity of the merkle tree
        assert_eq!(WIDTH, MERKLE_ARITY + 1);

        // Grant the desired arity is consitent to the desired width
        assert_eq!(MERKLE_ARITY.pow(MERKLE_HEIGHT as u32 - 1), MERKLE_WIDTH);
    }
}
