//! This module is designed to load from `ark.bin` the 960
//! constants used as `round_constants` in a `lazy_static` module.
//!
//! The constants were originally computed using:
//! https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py
//! and then mapped onto `DalekScalar` in the Ristretto scalar field.
#![allow(non_snake_case)]
use crate::Scalar;

use std::cmp;

use algebra::biginteger::BigInteger256;
use lazy_static::lazy_static;
use num_traits::{One, Zero};
use sha2::{Digest, Sha512};

const CONSTANTS: usize = 960;

lazy_static! {
  /// `ROUND_CONSTANTS` constists on a static reference
  /// that points to the pre-loaded 960 DalekScalar constants.
  ///
  /// This 960 `DalekScalar` constants are loaded from `ark.bin`
  /// where all of the `DalekScalar` are represented in bytes.
  ///
  /// This round constants have been taken from:
  /// https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py
  /// and then mapped onto `DalekScalar` in the Ristretto scalar field.
  pub static ref ROUND_CONSTANTS: [Scalar; CONSTANTS] = {
      constants()
  };
}

fn slice_to_u64(bytes: &[u8]) -> u64 {
    let mut s = [0x00u8; 8];
    let chunk = cmp::min(bytes.len(), 8);

    (&mut s[0..chunk]).copy_from_slice(&bytes[0..chunk]);

    u64::from_be_bytes(s)
}

fn constants() -> [Scalar; 960] {
    // TODO - Review constants generation
    let mut cnst = [Scalar::zero(); 960];
    let mut p = Scalar::one();
    let mut bytes = b"poseidon-for-plonk".to_vec();
    let two = Scalar::from(2u64);

    (0..CONSTANTS).for_each(|i| {
        let mut hasher = Sha512::new();
        hasher.input(bytes.as_slice());
        bytes = hasher.result().to_vec();

        let mut v = [0x00u8; 64];
        v.copy_from_slice(&bytes[0..64]);

        let a = slice_to_u64(&bytes[0..]);
        let b = slice_to_u64(&bytes[8..]);
        let c = slice_to_u64(&bytes[16..]);
        let d = slice_to_u64(&bytes[24..]);
        let s = Scalar::from(BigInteger256([a, b, c, d]));

        cnst[i] = s + p / two;
        p = cnst[i];
    });

    cnst
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_round_constants() {
        // Check each element is non-zero
        let zero = Scalar::zero();
        let has_zero = ROUND_CONSTANTS.iter().any(|&x| x == zero);
        assert!(!has_zero);
    }
}
