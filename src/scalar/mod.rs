use crate::errors::PermError;
use crate::mds_matrix::MDS_MATRIX;
use crate::round_constants::ROUND_CONSTANTS;
use curve25519_dalek::scalar::Scalar;

const TOTAL_FULL_ROUNDS: usize = 8;
const PARTIAL_ROUNDS: usize = 59;

pub struct Permutation {
  // data to be used in the solid instantiation of the permutation struct
  pub(crate) data: Vec<Scalar>,
}

impl Default for Permutation {
  fn default() -> Self {
    Self {
      data: Vec::with_capacity(MDS_MATRIX.len()),
    }
  }
}

impl Permutation {
  #[allow(non_snake_case)]
  pub fn new() -> Self {
    Self {
      data: Vec::with_capacity(MDS_MATRIX.len()),
    }
  }
}

// Utility methods on the permutation struct
impl Permutation {
  pub fn input(&mut self, scalar: Scalar) -> Result<(), PermError> {
    if self.data.len() == MDS_MATRIX.len() {
      return Err(PermError::InputFull);
    }
    self.data.push(scalar);
    Ok(())
  }
}

impl Permutation {
  pub fn result(&mut self) -> Result<Vec<Scalar>, PermError> {
    // Pad remaining width with zero
    self.data.resize_with(MDS_MATRIX.len(), Scalar::zero);

    let mut constants_iter = ROUND_CONSTANTS.iter();

    let mut new_words: Vec<Scalar> = self.data.clone();

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

// Apply partial rounds
impl Permutation {
  fn apply_partial_round<'a, I>(
    &self,
    constants: &mut I,
    words: Vec<Scalar>,
  ) -> Result<Vec<Scalar>, PermError>
  where
    I: Iterator<Item = &'a Scalar>,
  {
    // Add round keys to each word
    let mut new_words = self.add_round_key(constants, words)?;
    // Then apply quintic s-box to first element
    new_words[0] = Self::quintic_s_box(&new_words[0]);
    // Multiply this result by the MDS matrix
    Ok(new_words * &MDS_MATRIX)
  }
}

// Apply full round
impl Permutation {
  fn apply_full_round<'a, I>(
    &self,
    constants: &mut I,
    words: Vec<Scalar>,
  ) -> Result<Vec<Scalar>, PermError>
  where
    I: Iterator<Item = &'a Scalar>,
  {
    // Add round keys to each word
    let new_words = self.add_round_key(constants, words)?;

    // Then apply quintic s-box
    let quintic_words: Result<Vec<Scalar>, PermError> = new_words
      .iter()
      .map(|word| Ok(Self::quintic_s_box(word)))
      .collect();

    // Multiply this result by the MDS matrix
    Ok(quintic_words? * &MDS_MATRIX)
  }
}

// Add round key
impl Permutation {
  fn add_round_key<'a, I>(
    &self,
    constants: &mut I,
    words: Vec<Scalar>,
  ) -> Result<Vec<Scalar>, PermError>
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
}

impl Permutation {
  fn quintic_s_box(scalar: &Scalar) -> Scalar {
    scalar * scalar * scalar * scalar * scalar
  }
}

pub fn hash(mut perm: Permutation) -> Result<Scalar, PermError> {
  let words = perm.result()?;
  Ok(words[1])
}
