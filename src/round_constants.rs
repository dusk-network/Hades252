//! This module is designed to load from `ark.bin` the 960
//! constants used as `round_constants` in a `lazy_static` module.
//!
//! The constants were originally computed using:
//! https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py
//! and then mapped onto `Scalar` in the Ristretto scalar field.
#![allow(non_snake_case)]

use crate::{PARTIAL_ROUNDS, TOTAL_FULL_ROUNDS, WIDTH};
use curve25519_dalek::scalar::Scalar;
use lazy_static::*;

lazy_static! {
  /// `ROUND_CONSTANTS` constists on a static reference
  /// that points to the pre-loaded 960 Scalar constants.
  ///
  /// This 960 `Scalar` constants are loaded from `ark.bin`
  /// where all of the `Scalar` are represented in bytes.
  ///
  /// This round constants have been taken from:
  /// https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py
  /// and then mapped onto `Scalar` in the Ristretto scalar field.
  pub static ref ROUND_CONSTANTS: [Scalar; 960] = {
    let bytes = include_bytes!("../assets/ark.bin");

    assert!(bytes.len() >= ((TOTAL_FULL_ROUNDS + PARTIAL_ROUNDS) * WIDTH) << 5);
    unsafe { std::ptr::read(bytes.as_ptr() as *const _) }
  };
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
