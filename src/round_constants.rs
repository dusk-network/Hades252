#![allow(non_snake_case)]

use curve25519_dalek::scalar::Scalar;
use sha2::{Digest, Sha512};

pub struct RoundConstants {
    constants: Vec<Scalar>,
}

impl RoundConstants {
    pub fn iter<'a>(&'a self) -> RoundConstantsIterator<'a> {
        RoundConstantsIterator::new(&self.constants)
    }
    // This method differs from grain LFSR.
    // as mentioned in the paper pg13, other methods can be used.
    // We seed a sha512 hash function, then use hash_to_group to fetch a scalar in Z/pZ
    // note that both sha512 and hash_to_group are deterministic
    pub fn generate(num_full_rounds: usize, num_partial_rounds: usize, t: usize) -> Self {
        let total_num_constants = (num_full_rounds + num_partial_rounds) * t;
        let mut round_constants: Vec<Scalar> = Vec::with_capacity(total_num_constants);

        let mut hasher = Sha512::default();
        hasher.input(b"hades252_full_rounds");

        // First generate the constants for the full rounds
        for _ in 0..(num_full_rounds * t) {
            let round_constant = Scalar::from_hash(hasher.clone());
            round_constants.push(round_constant);

            hasher.reset();
            hasher.input(round_constant.as_bytes());
        }

        hasher.reset();
        hasher.input(b"hades252_partial_rounds");

        // Generate the constants for the partial rounds
        for _ in 0..(num_partial_rounds * t) {
            let round_constant = Scalar::from_hash(hasher.clone());
            round_constants.push(round_constant);

            hasher.reset();
            hasher.input(round_constant.as_bytes());
        }

        RoundConstants {
            constants: round_constants,
        }
    }
}

pub struct RoundConstantsIterator<'a> {
    index: usize,
    end_index: usize,
    constants: &'a Vec<Scalar>,
}

impl<'a> RoundConstantsIterator<'a> {
    pub fn new(constants: &'a Vec<Scalar>) -> RoundConstantsIterator<'a> {
        RoundConstantsIterator {
            index: 0,
            end_index: constants.len() - 1,
            constants: constants,
        }
    }
}

impl<'a> Iterator for RoundConstantsIterator<'a> {
    type Item = &'a Scalar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.end_index {
            return None;
        }

        let scalar = &self.constants[self.index];
        self.index = self.index + 1;

        Some(scalar)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_round_constants() {
        let num_full_rounds = 8;
        let num_partial_rounds = 127;
        let t = 6;

        let expected_num_constants = (num_full_rounds + num_partial_rounds) * t;

        let round_constants = RoundConstants::generate(num_full_rounds, num_partial_rounds, t);
        assert_eq!(expected_num_constants, round_constants.constants.len());

        // Check each element is non-zero
        let non_zero_element = round_constants
            .constants
            .iter()
            .find(|&&x| x == Scalar::zero());
        assert!(non_zero_element.is_none());

        let mut constants_iter = round_constants.iter();
        for _ in 0..expected_num_constants {
            let x = constants_iter.next();

            assert!(x.is_some());
            assert_ne!(&Scalar::zero(), x.unwrap());
        }
    }
}
