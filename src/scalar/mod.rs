//! This module contains an implementation of the `Poseidon252`
//! hash algorithm specifically designed to work outside of
//! Rank 1 Constraint Systems (R1CS).
//! 
//! The inputs of the hash function have to be explicitly
//! over the Scalar Field of the curve25519 so working over
//! `Fp = 2^252 + 27742317777372353535851937790883648493`.
use crate::errors::PermError;
use crate::mds_matrix::MDS_MATRIX;
use crate::round_constants::ROUND_CONSTANTS;
use curve25519_dalek::scalar::Scalar;

/// Total ammount of full rounds that will be applied. 
/// This is expressed as `RF` in the Poseidon paper.
const TOTAL_FULL_ROUNDS: usize = 8;

/// Total ammount of partial rounds that will be applied. 
/// This is expressed as `Rp` in the Poseidon paper.
const PARTIAL_ROUNDS: usize = 59;

/// Applies a `permutation-round` of the `Poseidon252` hashing algorithm. 
/// 
/// It returns a vec of `WIDTH` outputs as a result which should be 
/// a randomly permuted version of the input.  
/// 
/// In general, the same round function is iterated enough times
/// to make sure that any symmetries and structural properties that
/// might exist in the round function vanish.
/// 
/// This `permutation` is a 3-step process that:
/// 
/// - Applies twice the half of the `FULL_ROUNDS` 
/// (which can be understood as linear ops).
///  
/// - In the middle step it applies the `PARTIAL_ROUDS` 
/// (which can be understood as non-linear ops).
/// 
/// This structure allows to minimize the number of non-linear
/// ops while mantaining the security.
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

/// Applies a `Partial Round` also known as a 
/// `Partial S-Box layer` to a set of inputs. 
/// 
/// ### A partial round has 3 steps on every iteration:
/// 
/// - Add round keys to each word. Also known as `ARK`.
/// - Apply `quintic S-Box` **just to the first element of 
/// the words generated from the first step.** This is also known
/// as a `Sub Words` operation.
/// - Multiplies the output words from the second step by
/// the `MDS_MATRIX`.
/// This is known as the `Mix Layer`.
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

/// Applies a `Full Round` also known as a 
/// `Full S-Box layer` to a set of inputs. 
/// 
/// A full round has 3 steps on every iteration:
/// 
/// - Add round keys to each word. Also known as `ARK`.
/// - Apply `quintic S-Box` **to all of the words generated 
/// from the first step.** 
/// This is also known as a `Sub Words` operation.
/// - Multiplies the output words from the second step by
/// the `MDS_MATRIX`.
/// This is known as the `Mix Layer`.
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

/// Add round keys to a set of `Scalar`. 
/// 
/// This round key addition also known as `ARK` is used to
/// reach `Confusion and Diffusion` properties for the algorithm.
/// 
/// Basically it allows to destroy any connection between the 
/// inputs and the outputs of the function.
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

/// Computes `input ^ 5 (mod Fp)`
/// 
/// The modulo depends on the input you use. In our case
/// the modulo is done in respect of the `curve25519 scalar field`
///  == `2^252 + 27742317777372353535851937790883648493`.
fn quintic_s_box(scalar: &Scalar) -> Scalar {
  scalar * scalar * scalar * scalar * scalar
}

/// Performs the Poseidon-252 hash algorithm over a set of inputs. 
/// 
/// In this implementation, apply the hash is the same as applying
/// just one permutation over the inputs (padding and setting the 
/// bitflags first) since the arity of the merkle tree is `9` 
/// and we don't accept more than 8 inputs. 
/// 
/// # Returns
/// - `Ok(Scalar)` if the number of inputs is lower than `8`. 
/// - `Err` -> `PermError`: Which means that the ammount of inputs
/// of the hash function exceeds the limit `8`.
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


  let words = perm(words)?;
  Ok(words[1])
}