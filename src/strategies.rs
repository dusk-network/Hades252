//! This module contains an implementation of the `Hades252`
//! strategy algorithm specifically designed to work outside of
//! Rank 1 Constraint Systems (R1CS).
//!
//! The inputs of the permutation function have to be explicitly
//! over the Scalar Field of the curve25519 so working over
//! `Fp = 2^252 + 27742317777372353535851937790883648493`.
use crate::mds_matrix::MDS_MATRIX;
use crate::round_constants::ROUND_CONSTANTS;
use crate::{PARTIAL_ROUNDS, TOTAL_FULL_ROUNDS};
use bulletproofs::r1cs::{ConstraintSystem, LinearCombination};
use curve25519_dalek::scalar::Scalar;
use std::iter::FromIterator;
use std::ops::Add;

pub trait StrategyInput = From<Scalar> + Clone + Add;

/// Defines the Hades252 strategy algorithm.
pub trait Strategy<T>
where
    T: StrategyInput,
    Vec<T>: FromIterator<<T as Add>::Output>,
{
    /// Computes `input ^ 5 (mod Fp)`
    ///
    /// The modulo depends on the input you use. In our case
    /// the modulo is done in respect of the `curve25519 scalar field`
    ///  == `2^252 + 27742317777372353535851937790883648493`.
    fn quintic_s_box(&mut self, value: &T) -> T;

    /// Multiply the values for MDS matrix.
    fn mul_matrix(&mut self, values: Vec<T>) -> Vec<T>;

    /// Add round keys to a set of `StrategyInput`.
    ///
    /// This round key addition also known as `ARK` is used to
    /// reach `Confusion and Diffusion` properties for the algorithm.
    ///
    /// Basically it allows to destroy any connection between the
    /// inputs and the outputs of the function.
    fn add_round_key<'a, I>(&mut self, constants: &mut I, words: Vec<T>) -> Vec<T>
    where
        I: Iterator<Item = &'a Scalar>,
    {
        words
            .iter()
            .map(|word| {
                let c = constants.next().unwrap();
                let c = T::from(*c);
                word.clone() + c
            })
            .collect()
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
    fn apply_partial_round<'a, I>(&mut self, constants: &mut I, words: Vec<T>) -> Vec<T>
    where
        I: Iterator<Item = &'a Scalar>,
    {
        // Add round keys to each word
        let mut new_words = self.add_round_key(constants, words);
        // Then apply quintic s-box to first element
        new_words[0] = self.quintic_s_box(&new_words[0]);
        // Multiply this result by the MDS matrix
        self.mul_matrix(new_words)
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
    fn apply_full_round<'a, I>(&mut self, constants: &mut I, words: Vec<T>) -> Vec<T>
    where
        I: Iterator<Item = &'a Scalar>,
    {
        // Add round keys to each word
        let new_words = self.add_round_key(constants, words);

        // Then apply quintic s-box
        let quintic_words: Vec<T> = new_words
            .iter()
            .map(|word| self.quintic_s_box(word))
            .collect();

        // Multiply this result by the MDS matrix
        self.mul_matrix(quintic_words)
    }

    /// Applies a `permutation-round` of the `Hades252` strategy.
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
    fn perm(&mut self, data: Vec<T>) -> Vec<T> {
        let mut constants_iter = ROUND_CONSTANTS.iter();

        let mut new_words = data;

        // Apply R_f full rounds
        for _ in 0..TOTAL_FULL_ROUNDS / 2 {
            new_words = self.apply_full_round(&mut constants_iter, new_words);
        }

        // Apply R_P partial rounds
        for _ in 0..PARTIAL_ROUNDS {
            new_words = self.apply_partial_round(&mut constants_iter, new_words);
        }

        // Apply R_f full rounds
        for _ in 0..TOTAL_FULL_ROUNDS / 2 {
            new_words = self.apply_full_round(&mut constants_iter, new_words);
        }

        new_words
    }
}

/// Implements a Hades252 strategy for `Scalar` as input values.
#[derive(Default)]
pub struct ScalarStrategy {}

impl ScalarStrategy {
    /// Constructs a new `ScalarStrategy`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Strategy<Scalar> for ScalarStrategy {
    fn quintic_s_box(&mut self, value: &Scalar) -> Scalar {
        value * value * value * value * value
    }
    fn mul_matrix(&mut self, values: Vec<Scalar>) -> Vec<Scalar> {
        values * &MDS_MATRIX
    }
}

/// Implements a Hades252 strategy for `LinearCombination` as input values.
/// Requires a reference to a `ConstraintSystem`.
pub struct GadgetStrategy<'a> {
    /// A reference to the constraint system used by the gadgets
    pub cs: &'a mut dyn ConstraintSystem,
}

impl<'a> GadgetStrategy<'a> {
    /// Constructs a new `GadgetStrategy` with the constraint system.
    pub fn new(cs: &'a mut dyn ConstraintSystem) -> Self {
        GadgetStrategy { cs }
    }
}
impl<'a> Strategy<LinearCombination> for GadgetStrategy<'a> {
    fn quintic_s_box(&mut self, value: &LinearCombination) -> LinearCombination {
        let (value, _, square) = self.cs.multiply(value.clone(), value.clone());
        let (_, _, quartic) = self.cs.multiply(square.into(), square.into());
        let (_, _, quintic) = self.cs.multiply(quartic.into(), value.into());

        quintic.into()
    }
    fn mul_matrix(&mut self, values: Vec<LinearCombination>) -> Vec<LinearCombination> {
        values * &MDS_MATRIX
    }
}
