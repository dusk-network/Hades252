use crate::errors::PermError;
use crate::mds_matrix::MDS_MATRIX;
use crate::round_constants::ROUND_CONSTANTS;
use bulletproofs::r1cs::{ConstraintSystem, LinearCombination};
use curve25519_dalek::scalar::Scalar;

const TOTAL_FULL_ROUNDS: usize = 8;
const PARTIAL_ROUNDS: usize = 59;

pub struct Permutation<'a> {
  // data to be used in the solid instantiation of the permutation struct
  pub(crate) data: Vec<LinearCombination>,
  pub(crate) cs: &'a mut dyn ConstraintSystem,
}

impl<'a> Permutation<'a> {
  #[allow(non_snake_case)]
  pub fn new(cs: &'a mut dyn ConstraintSystem) -> Self {
    Self {
      data: Vec::with_capacity(MDS_MATRIX.len()),
      cs,
    }
  }
}

// Utility methods on the permutation struct
impl Permutation<'_> {
  pub fn input(&mut self, lc: LinearCombination) -> Result<(), PermError> {
    if self.data.len() == MDS_MATRIX.len() {
      return Err(PermError::InputFull);
    }
    self.data.push(lc);
    Ok(())
  }
}

impl Permutation<'_> {
  pub fn result(&mut self) -> Result<Vec<LinearCombination>, PermError> {
    // Pad remaining width with zero
    self.data.resize_with(MDS_MATRIX.len(), || Scalar::zero().into());

    let mut constants_iter = ROUND_CONSTANTS.iter();

    let mut new_words: Vec<LinearCombination> = self.data.clone();

    // Apply R_f full rounds
    for _ in 0..TOTAL_FULL_ROUNDS / 2 {
      new_words = self.apply_full_round(&mut constants_iter, new_words)?;
    }

    // Apply R_P partial rounds
    for _ in 0..PARTIAL_ROUNDS {
      new_words = self.apply_partial_round(&mut constants_iter, new_words)?;
    }

    // Apply R_f full rounds
    for _ in 0..TOTAL_FULL_ROUNDS / 2 {
      new_words = self.apply_full_round(&mut constants_iter, new_words)?;
    }

    Ok(new_words)
  }
}

// Rounds
impl Permutation<'_> {
  fn apply_partial_round<'a, I>(
    &mut self,
    constants: &mut I,
    words: Vec<LinearCombination>,
  ) -> Result<Vec<LinearCombination>, PermError>
  where
    I: Iterator<Item = &'a Scalar>,
  {
    // Add round keys to each word
    let mut new_words = self.add_round_key(constants, words)?;
    // Then apply quintic s-box to first element
    new_words[0] = self.quintic_s_box(new_words[0].clone());
    // Multiply this result by the MDS matrix
    Ok(new_words * &MDS_MATRIX)
  }

  fn apply_full_round<'a, I>(
    &mut self,
    constants: &mut I,
    words: Vec<LinearCombination>,
  ) -> Result<Vec<LinearCombination>, PermError>
  where
    I: Iterator<Item = &'a Scalar>,
  {
    // Add round keys to each word
    let new_words = self.add_round_key(constants, words)?;

    // Then apply quintic s-box
    let quintic_words: Result<Vec<LinearCombination>, PermError> = new_words
      .iter()
      .map(|word| Ok(self.quintic_s_box(word.clone())))
      .collect();

    // Multiply this result by the MDS matrix
    Ok(quintic_words? * &MDS_MATRIX)
  }
}

// Add round key
impl Permutation<'_> {
  fn add_round_key<'b, I>(
    &self,
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
}

impl Permutation<'_> {
  fn quintic_s_box(&mut self, lc: LinearCombination) -> LinearCombination {
    let (lc, _, square) = self.cs.multiply(lc.clone(), lc);
    let (_, _, quartic) = self.cs.multiply(square.into(), square.into());
    let (_, _, quintic) = self.cs.multiply(quartic.into(), lc.into());

    quintic.into()
  }
}

pub fn hash<'a>(mut perm: Permutation) -> Result<LinearCombination, PermError> {
  let words = perm.result()?;
  Ok(words[1].clone())
}
