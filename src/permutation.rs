use crate::errors::PermError;
use crate::mds_matrix::*;
use crate::round_constants::*;
use bulletproofs::r1cs::{ConstraintSystem, LinearCombination, Variable};
use curve25519_dalek::scalar::Scalar;
use sha2::Sha512;

pub struct Permutation {
    t: usize,
    full_rounds: usize,
    partial_rounds: usize,

    // data to be used in the solid instantiation of the permutation struct
    pub(crate) data: Vec<Scalar>,
    // data to be used in the constraint system instantiation of the permutation struct
    pub(crate) data_lc: Vec<LinearCombination>,

    constants: RoundConstants,
    matrix: MDSMatrix,
}

impl Default for Permutation {
    fn default() -> Self {
        let width = 9;
        let total_full_founds = 8;
        let partial_rounds = 59;
        Permutation {
            t: width,
            full_rounds: full_founds,
            partial_rounds: partial_rounds,
            data: Vec::with_capacity(width),
            data_lc: Vec::with_capacity(width),
            constants: RoundConstants::generate(full_founds, partial_rounds, width),
            matrix: MDSMatrix::generate(width),
        }
    }
}

impl Permutation {
    #[allow(non_snake_case)]
    pub fn new(t: usize, R_f: usize, R_p: usize) -> Self {
        let total_full_rounds = 2 * R_f;

        Permutation {
            t: t,
            full_rounds: total_full_rounds,
            partial_rounds: R_p,
            data: Vec::with_capacity(t),
            data_lc: Vec::with_capacity(t),
            constants: RoundConstants::generate(total_full_rounds, R_p, t),
            matrix: MDSMatrix::generate(t),
        }
    }
}

// Utility methods on the permutation struct
impl Permutation {
    pub fn reset(&mut self) {
        self.data.clear();
        self.data_lc.clear();
    }
    fn input_full<T>(&self, data: &Vec<T>) -> bool {
        data.len() == self.t
    }
    pub fn width_left<T>(&self, data: &Vec<T>) -> usize {
        self.t - data.len()
    }
    pub fn input_bytes(&mut self, bytes: &[u8]) -> Result<(), PermError> {
        // Map arbitrary bytes to group using elligator2
        let scalar = Scalar::hash_from_bytes::<Sha512>(bytes);
        self.input(scalar)
    }
    pub fn input(&mut self, scalar: Scalar) -> Result<(), PermError> {
        if self.input_full(&self.data) {
            return Err(PermError::InputFull);
        }
        self.data.push(scalar);
        Ok(())
    }
    pub fn inputs(&mut self, scalars: Vec<Scalar>) -> Result<(), PermError> {
        let amount_to_add = scalars.len();
        let maximum_width = self.t;
        let current_width = self.data.len();

        if amount_to_add + current_width > maximum_width {
            return Err(PermError::InputFull);
        }

        self.data.extend(scalars);
        Ok(())
    }
    pub fn input_lc(&mut self, lc: LinearCombination) -> Result<(), PermError> {
        if self.input_full(&self.data_lc) {
            return Err(PermError::InputFull);
        }
        self.data_lc.push(lc);
        Ok(())
    }
    fn pad(&mut self) {
        let pad_amount = self.width_left(&self.data);
        let zero = Scalar::zero();
        let zeroes = vec![zero; pad_amount];

        self.data.extend(zeroes);
    }
    fn pad_lc(&mut self) {
        let pad_amount = self.width_left(&self.data_lc);
        let zero_lc: LinearCombination = Scalar::zero().into();
        let zeroes = vec![zero_lc; pad_amount];

        self.data_lc.extend(zeroes);
    }
}

impl Permutation {
    pub fn result(&mut self) -> Result<Vec<Scalar>, PermError> {
        // Pad remaining width with zero
        self.pad();

        let fulll_rounds_iter = self.full_rounds / 2;

        let mut constants_iter = self.constants.iter();

        let mut new_words: Vec<Scalar> = self.data.clone();

        // Apply R_f full rounds
        for _ in 0..fulll_rounds_iter {
            new_words = self.apply_full_round(&mut constants_iter, new_words)?;
        }

        // Apply R_P partial rounds
        for _ in 0..self.partial_rounds {
            new_words = self.apply_partial_round(&mut constants_iter, new_words)?;
        }

        // Apply R_f full rounds
        for _ in 0..fulll_rounds_iter {
            new_words = self.apply_full_round(&mut constants_iter, new_words)?;
        }

        Ok(new_words)
    }

    pub fn constrain_result(
        &mut self,
        cs: &mut dyn ConstraintSystem,
    ) -> Result<Vec<LinearCombination>, PermError> {
        // Pad remaining width with zero
        self.pad_lc();

        let mut constants_iter = self.constants.iter();

        let mut new_words = self.data_lc.clone();

        let fulll_rounds_iter = self.full_rounds / 2;

        // Apply R_f full rounds
        for _ in 0..fulll_rounds_iter{
            new_words = self.constrain_apply_full_round(&mut constants_iter, new_words, cs)?;
            new_words = new_words.into_iter().map(|word| word.simplify()).collect();
        }

        // Apply R_P partial rounds
        for _ in 0..self.partial_rounds {
            new_words = self.constrain_apply_partial_round(&mut constants_iter, new_words, cs)?;
            new_words = new_words.into_iter().map(|word| word.simplify()).collect();
        }

        // Apply R_f full rounds
        for _ in 0..fulll_rounds_iter{
            new_words = self.constrain_apply_full_round(&mut constants_iter, new_words, cs)?;
            new_words = new_words.into_iter().map(|word| word.simplify()).collect();
        }

        Ok(new_words)
    }
}

// Apply partial rounds
impl Permutation {
    fn apply_partial_round(
        &self,
        constants: &mut RoundConstantsIterator,
        words: Vec<Scalar>,
    ) -> Result<Vec<Scalar>, PermError> {
        // Add round keys to each word
        let mut new_words = self.add_round_key(constants, words)?;
        // Then apply quintic s-box to first element
        new_words[0] = Permutation::quintic_s_box(&new_words[0]);
        // Multiply this result by the MDS matrix
        Ok(self.matrix.mul_vector(&new_words))
    }
    fn constrain_apply_partial_round(
        &self,
        constants: &mut RoundConstantsIterator,
        words: Vec<LinearCombination>,
        cs: &mut dyn ConstraintSystem,
    ) -> Result<Vec<LinearCombination>, PermError> {
        // Add round keys to each word
        let mut new_words = self.constrain_add_round_key(constants, words)?;
        // Then apply quintic s-box to first element
        new_words[0] = Permutation::constrain_quintic_s_box(new_words[0].clone(), cs);
        // Multiply this result by the MDS matrix
        Ok(self.matrix.constrain_mul_vector(new_words))
    }
}

// Apply full round
impl Permutation {
    fn apply_full_round(
        &self,
        constants: &mut RoundConstantsIterator,
        words: Vec<Scalar>,
    ) -> Result<Vec<Scalar>, PermError> {
        // Add round keys to each word
        let new_words = self.add_round_key(constants, words)?;

        // Then apply quintic s-box
        let quintic_words: Result<Vec<Scalar>, PermError> = new_words
            .iter()
            .map(|word| Ok(Permutation::quintic_s_box(word)))
            .collect();

        // Multiply this result by the MDS matrix
        Ok(self.matrix.mul_vector(&quintic_words?))
    }

    fn constrain_apply_full_round(
        &self,
        constants: &mut RoundConstantsIterator,
        words: Vec<LinearCombination>,
        cs: &mut dyn ConstraintSystem,
    ) -> Result<Vec<LinearCombination>, PermError> {
        // Add round keys to each word
        let new_words = self.constrain_add_round_key(constants, words)?;

        let quintic_words: Result<Vec<LinearCombination>, PermError> = new_words
            .iter()
            .map(|word| Ok(Permutation::constrain_quintic_s_box(word.clone(), cs)))
            .collect();

        // Multiply this result by the MDS matrix
        Ok(self.matrix.constrain_mul_vector(quintic_words?))
    }
}

// Add round key
impl Permutation {
    fn add_round_key(
        &self,
        constants: &mut RoundConstantsIterator,
        words: Vec<Scalar>,
    ) -> Result<Vec<Scalar>, PermError> {
        words
            .iter()
            .map(|word| {
                let c = constants.next().ok_or(PermError::NoMoreConstants)?;
                Ok(word + c)
            })
            .collect()
    }

    fn constrain_add_round_key(
        &self,
        constants: &mut RoundConstantsIterator,
        words: Vec<LinearCombination>,
    ) -> Result<Vec<LinearCombination>, PermError> {
        words
            .iter()
            .map(|word| {
                // First get the constant needed and convert it to a linear combination
                let c = constants.next().ok_or(PermError::NoMoreConstants)?;
                let c_lc = LinearCombination::from(c.clone());
                // Return words + constants
                Ok(word.clone() + c_lc)
            })
            .collect()
    }
}

impl Permutation {
    fn quintic_s_box(scalar: &Scalar) -> Scalar {
        scalar * scalar * scalar * scalar * scalar
    }
    fn constrain_quintic_s_box(
        lc: LinearCombination,
        cs: &mut dyn ConstraintSystem,
    ) -> LinearCombination {
        let (lc, _, square) = cs.multiply(lc.clone(), lc);
        let (_, _, quartic) = cs.multiply(square.into(), square.into());
        let (_, _, quintic) = cs.multiply(quartic.into(), lc.into());

        quintic.into()
    }
}
