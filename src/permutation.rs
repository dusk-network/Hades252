use crate::errors::PermError;
use crate::mds_matrix::*;
use crate::round_constants::*;
use curve25519_dalek::scalar::Scalar;
use sha2::Sha512;

pub struct Permutation {
    t: usize,
    full_rounds: usize,
    partial_rounds: usize,

    pub(crate) data: Vec<Scalar>,

    constants: RoundConstants,
    matrix: MDSMatrix,
}

impl Default for Permutation {
    fn default() -> Self {
        let width = 6;
        let full_founds = 8;
        let partial_rounds = 127;
        Permutation {
            t: width,
            full_rounds: full_founds,
            partial_rounds: partial_rounds,
            data: Vec::with_capacity(width),
            constants: RoundConstants::generate(full_founds, partial_rounds, width),
            matrix: MDSMatrix::generate(width),
        }
    }
}

impl Permutation {
    pub fn new(t: usize, full_rounds: usize, partial_rounds: usize) -> Result<Self, PermError> {
        // We could ask for R_f instead of R_F then multiply by two.
        // It would make for a better API, however would need to be documented correctly
        // Because partial rounds means "everything", while full_rounds would mean "half of the full rounds"
        if full_rounds % 2 != 0 {
            return Err(PermError::FullRoundsOdd);
        }

        let perm = Permutation {
            t: t,
            full_rounds: full_rounds,
            partial_rounds: partial_rounds,
            data: Vec::with_capacity(t),
            constants: RoundConstants::generate(full_rounds, partial_rounds, t),
            matrix: MDSMatrix::generate(t),
        };

        Ok(perm)
    }
    pub fn reset(&mut self) {
        self.data.clear()
    }
    fn input_full(&self) -> bool {
        self.data.len() == self.t
    }
    pub fn width_left(&self) -> usize {
        self.t - self.data.len()
    }
    pub fn input_bytes(&mut self, bytes: &[u8]) -> Result<(), PermError> {
        // Map bytes to group using elligator2
        let scalar = Scalar::hash_from_bytes::<Sha512>(bytes);
        self.input(scalar)
    }
    pub fn input(&mut self, scalar: Scalar) -> Result<(), PermError> {
        if self.input_full() {
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
    fn add_round_key(
        &self,
        constants: &mut RoundConstantsIterator,
        words : Vec<Scalar>,
    ) -> Result<Vec<Scalar>, PermError> {
        words
            .iter()
            .map(|word| {
                let c = constants.next().ok_or(PermError::NoMoreConstants)?;
                Ok(word + c)
            })
            .collect()
    }
    fn apply_full_round(
        &self,
        constants: &mut RoundConstantsIterator,
        words : Vec<Scalar>,
    ) -> Result<Vec<Scalar>, PermError> {
        // Add round keys to each word
        let new_words = self.add_round_key(constants, words)?;

        // Then apply inverse s-box by inverting the result
        let inverted_words: Result<Vec<Scalar>, PermError> = new_words
            .iter()
            .map(|word| {
                if word == &Scalar::zero() {
                    return Err(PermError::NonInvertible);
                }
                Ok(word.invert())
            })
            .collect();

        // Multiply this result by the MDS matrix
        Ok(self.matrix.mul_vector(&inverted_words?))
    }
    fn apply_partial_round(
        &self,
        constants: &mut RoundConstantsIterator,
        words : Vec<Scalar>,
    ) -> Result<Vec<Scalar>, PermError> {
        // Add round keys to each word
        let mut new_words = self.add_round_key(constants, words)?;
        // Then apply inversion s-box to first element
        if new_words[0] == Scalar::zero() {
            return Err(PermError::NonInvertible);
        }
        new_words[0] = new_words[0].invert();
        // Multiply this result by the MDS matrix
        Ok(self.matrix.mul_vector(&new_words))
    }

    pub fn result(&self) -> Result<Vec<Scalar>, PermError> {
        let mut constants_iter = self.constants.iter();

        let mut new_words: Vec<Scalar> = self.data.clone();

        // Apply R_f full rounds
        for _ in 0..self.full_rounds / 2 {
            new_words = self.apply_full_round(&mut constants_iter, new_words)?;
        }

        // Apply R_P partial rounds
        for _ in 0..self.partial_rounds {
            new_words = self.apply_partial_round(&mut constants_iter, new_words)?;
        }

        // Apply R_f full rounds
        for _ in 0..self.full_rounds / 2 {
            new_words = self.apply_full_round(&mut constants_iter, new_words)?;
        }

        Ok(new_words)
    }
}