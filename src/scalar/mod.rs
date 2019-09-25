use crate::errors::PermError;
use crate::mds_matrix::MDS_MATRIX;
use crate::round_constants::ROUND_CONSTANTS;
use curve25519_dalek::scalar::Scalar;

const TOTAL_FULL_ROUNDS: usize = 8;
const PARTIAL_ROUNDS: usize = 59;

// Utility methods on the permutation struct
fn perm(data: Vec<Scalar>) -> Result<Vec<Scalar>, PermError> {
  let mut constants_iter = ROUND_CONSTANTS.iter();

  let mut new_words = data;

  // Apply R_f full rounds
  for _ in 0..TOTAL_FULL_ROUNDS / 2 {
    new_words = apply_full_round(&mut constants_iter, new_words)?;
  }

  // Apply R_P partial rounds
  for _ in 0..PARTIAL_ROUNDS {
    new_words = apply_partial_round(&mut constants_iter, new_words)?;
  }

  // Apply R_f full rounds
  for _ in 0..TOTAL_FULL_ROUNDS / 2 {
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

pub fn hash(data: &[Scalar]) -> Result<Scalar, PermError> {
  let width = MDS_MATRIX.len();

  if data.len() >= width - 1 {
    return Err(PermError::InputFull);
  }

  let mut words = vec![Scalar::zero(); width];
  let words_slice = &mut words[1..1 + data.len()];

  words_slice.copy_from_slice(data);

  let words = perm(words)?;
  Ok(words[1])
}
