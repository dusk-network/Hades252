//! This module is designed to load from `ark.bin` the 960
//! constants used as `round_constants` in a `lazy_static` module.
//!
//! The constants were originally computed using:
//! https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py
//! and then mapped onto `Scalar` in the Ristretto scalar field.
#![allow(non_snake_case)]
use crate::BlsScalar;

use lazy_static::lazy_static;

const CONSTANTS: usize = 960;

lazy_static! {
  /// `ROUND_CONSTANTS` constists on a static reference
  /// that points to the pre-loaded 960 Fq constants.
  ///
  /// This 960 `Fq` constants are loaded from `ark.bin`
  /// where all of the `Fq` are represented in bytes.
  ///
  /// This round constants have been taken from:
  /// https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py
  /// and then mapped onto `Fq` in the Ristretto scalar field.
  pub static ref ROUND_CONSTANTS: [BlsScalar; CONSTANTS] = {
      let bytes = include_bytes!("../assets/ark.bin");
      let mut a = [0x00u8; 8];
      let mut b = [0x00u8; 8];
      let mut c = [0x00u8; 8];
      let mut d = [0x00u8; 8];

      let mut cnst = [BlsScalar::zero(); CONSTANTS];
      cnst.iter_mut().zip((0..bytes.len()).step_by(32)).for_each(|(cn, i)| {
          a.copy_from_slice(&bytes[i..i+8]);
          b.copy_from_slice(&bytes[i+8..i+16]);
          c.copy_from_slice(&bytes[i+16..i+24]);
          d.copy_from_slice(&bytes[i+24..i+32]);

          *cn = BlsScalar::from_raw([
                  u64::from_le_bytes(a),
                  u64::from_le_bytes(b),
                  u64::from_le_bytes(c),
                  u64::from_le_bytes(d)
              ]);
      });

      cnst
  };
}

#[cfg(test)]
mod test {
    use super::ROUND_CONSTANTS;
    use crate::BlsScalar;

    #[test]
    fn test_round_constants() {
        // Check each element is non-zero
        let zero = BlsScalar::zero();
        let has_zero = ROUND_CONSTANTS.iter().any(|&x| x == zero);
        assert!(!has_zero);
    }
}
