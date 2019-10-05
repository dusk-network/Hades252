#![feature(test)]

// Poseidon constants
pub(crate) const WIDTH: usize = 5;
pub(crate) const FULL_ROUNDS: usize = 8;
pub(crate) const PARTIAL_ROUNDS: usize = 59;

// Merkle constants
pub(crate) const MERKLE_ARITY: usize = 4;
pub(crate) const _MERKLE_HEIGHT: usize = 4;
pub(crate) const MERKLE_WIDTH: usize = 64;
pub(crate) const MERKLE_INNER_WIDTH: usize = 80;

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
        // Grant we have enough constants for the sbox rounds
        assert!(WIDTH * (FULL_ROUNDS + PARTIAL_ROUNDS) <= round_constants::ROUND_CONSTANTS.len());

        // Sanity check for the arity
        assert!(MERKLE_ARITY > 1);

        // Sanity check for the height
        assert!(_MERKLE_HEIGHT > 2);

        // Enforce a relation between the provided MDS matrix and the arity of the merkle tree
        assert_eq!(WIDTH, MERKLE_ARITY + 1);

        // Enforce at least one level for the merkle tree
        assert!(MERKLE_WIDTH > MERKLE_ARITY);

        // Grant the defined arity is consistent with the defined width
        assert_eq!(
            MERKLE_ARITY.pow(std::cmp::max(2, _MERKLE_HEIGHT as u32 - 1)),
            MERKLE_WIDTH
        );

        // Grant the inner width is consistent to the proportion width x arity
        assert_eq!(WIDTH * MERKLE_WIDTH / MERKLE_ARITY, MERKLE_INNER_WIDTH)
    }
}
