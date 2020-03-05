//! This module is designed to load from `ark.bin` the 960
//! constants used as `round_constants` in a `lazy_static` module.
//!
//! The constants were originally computed using:
//! https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py
//! and then mapped onto `Scalar` in the Ristretto scalar field.
#![allow(non_snake_case)]
use crate::Scalar;

use lazy_static::lazy_static;

const CONSTANTS: usize = 960;

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
  pub static ref ROUND_CONSTANTS: [Scalar; CONSTANTS] = {
      let bytes = include_bytes!("../assets/ark.bin");
      let mut a = [0x00u8; 32];

      let mut cnst = [Scalar::zero(); CONSTANTS];

      cnst.iter_mut().zip((0..bytes.len()).step_by(32)).for_each(|(cn, i)| {
          a.copy_from_slice(&bytes[i..i+32]);
          let b = unsafe { std::mem::transmute_copy(&a) };
          *cn = Scalar::from_raw(b);
      });

      cnst
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
