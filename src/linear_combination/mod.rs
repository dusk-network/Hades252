use crate::errors::PermError;
use crate::mds_matrix::MDS_MATRIX;
use crate::round_constants::ROUND_CONSTANTS;
use crate::{FULL_ROUNDS, PARTIAL_ROUNDS};
use bulletproofs::r1cs::{ConstraintSystem, LinearCombination};
use curve25519_dalek::scalar::Scalar;

pub mod merkle;
pub mod sponge;

fn perm(
    cs: &mut dyn ConstraintSystem,
    data: Vec<LinearCombination>,
) -> Result<Vec<LinearCombination>, PermError> {
    let mut constants_iter = ROUND_CONSTANTS.iter();

    let mut new_words = data;

    // Apply R_f full rounds
    for _ in 0..FULL_ROUNDS / 2 {
        new_words = apply_full_round(cs, &mut constants_iter, new_words)?;
    }

    // Apply R_P partial rounds
    for _ in 0..PARTIAL_ROUNDS {
        new_words = apply_partial_round(cs, &mut constants_iter, new_words)?;
    }

    // Apply R_f full rounds
    for _ in 0..FULL_ROUNDS / 2 {
        new_words = apply_full_round(cs, &mut constants_iter, new_words)?;
    }

    Ok(new_words)
}

fn apply_partial_round<'a, I>(
    cs: &mut dyn ConstraintSystem,
    constants: &mut I,
    words: Vec<LinearCombination>,
) -> Result<Vec<LinearCombination>, PermError>
where
    I: Iterator<Item = &'a Scalar>,
{
    // Add round keys to each word
    let mut new_words = add_round_key(constants, words)?;
    // Then apply quintic s-box to first element
    new_words[0] = quintic_s_box(cs, new_words[0].clone());
    // Multiply this result by the MDS matrix
    Ok(new_words * &MDS_MATRIX)
}

fn apply_full_round<'a, I>(
    cs: &mut dyn ConstraintSystem,
    constants: &mut I,
    words: Vec<LinearCombination>,
) -> Result<Vec<LinearCombination>, PermError>
where
    I: Iterator<Item = &'a Scalar>,
{
    // Add round keys to each word
    let new_words = add_round_key(constants, words)?;

    // Then apply quintic s-box
    let quintic_words: Result<Vec<LinearCombination>, PermError> = new_words
        .iter()
        .map(|word| Ok(quintic_s_box(cs, word.clone())))
        .collect();

    // Multiply this result by the MDS matrix
    Ok(quintic_words? * &MDS_MATRIX)
}

// Add round key
fn add_round_key<'b, I>(
    constants: &mut I,
    words: Vec<LinearCombination>,
) -> Result<Vec<LinearCombination>, PermError>
where
    I: Iterator<Item = &'b Scalar>,
{
    words
        .iter()
        .map(|word| {
            let c = constants.next().ok_or(PermError::NoMoreConstants)?;
            let c = LinearCombination::from(*c);
            Ok(word.clone() + c)
        })
        .collect()
}

fn quintic_s_box(cs: &mut dyn ConstraintSystem, lc: LinearCombination) -> LinearCombination {
    let (lc, _, square) = cs.multiply(lc.clone(), lc);
    let (_, _, quartic) = cs.multiply(square.into(), square.into());
    let (_, _, quintic) = cs.multiply(quartic.into(), lc.into());

    quintic.into()
}
