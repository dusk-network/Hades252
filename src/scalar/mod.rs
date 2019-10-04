use crate::errors::PermError;
use crate::mds_matrix::MDS_MATRIX;
use crate::round_constants::ROUND_CONSTANTS;
use crate::{FULL_ROUNDS, PARTIAL_ROUNDS};

pub use curve25519_dalek::scalar::Scalar;

pub mod merkle;
pub mod sponge;

// Utility methods on the permutation struct
pub(crate) fn perm(data: Vec<Scalar>) -> Result<Vec<Scalar>, PermError> {
    let mut constants_iter = ROUND_CONSTANTS.iter();

    let mut new_words = data;

    // Apply R_f full rounds
    for _ in 0..FULL_ROUNDS / 2 {
        new_words = apply_full_round(&mut constants_iter, new_words)?;
    }

    // Apply R_P partial rounds
    for _ in 0..PARTIAL_ROUNDS {
        new_words = apply_partial_round(&mut constants_iter, new_words)?;
    }

    // Apply R_f full rounds
    for _ in 0..FULL_ROUNDS / 2 {
        new_words = apply_full_round(&mut constants_iter, new_words)?;
    }

    Ok(new_words)
}

fn apply_partial_round<'a, I>(
    constants: &mut I,
    words: Vec<Scalar>,
) -> Result<Vec<Scalar>, PermError>
where
    I: Iterator<Item = &'a Scalar>,
{
    // Add round keys to each word
    let mut new_words = add_round_key(constants, words)?;
    // Then apply quintic s-box to first element
    new_words[0] = quintic_s_box(&new_words[0]);
    // Multiply this result by the MDS matrix
    Ok(new_words * &MDS_MATRIX)
}

fn apply_full_round<'a, I>(constants: &mut I, words: Vec<Scalar>) -> Result<Vec<Scalar>, PermError>
where
    I: Iterator<Item = &'a Scalar>,
{
    // Add round keys to each word
    let new_words = add_round_key(constants, words)?;

    // Then apply quintic s-box
    let quintic_words: Result<Vec<Scalar>, PermError> = new_words
        .iter()
        .map(|word| Ok(quintic_s_box(word)))
        .collect();

    // Multiply this result by the MDS matrix
    Ok(quintic_words? * &MDS_MATRIX)
}

fn add_round_key<'a, I>(constants: &mut I, words: Vec<Scalar>) -> Result<Vec<Scalar>, PermError>
where
    I: Iterator<Item = &'a Scalar>,
{
    words
        .iter()
        .map(|word| {
            let c = constants.next().ok_or(PermError::NoMoreConstants)?;
            Ok(word + c)
        })
        .collect()
}

fn quintic_s_box(scalar: &Scalar) -> Scalar {
    scalar * scalar * scalar * scalar * scalar
}
