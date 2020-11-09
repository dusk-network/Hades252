// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! This module is designed to load from `ark.bin` the 960
//! constants used as `round_constants` in a `lazy_static` module.
//!
//! The constants were originally computed using:
//! https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py
//! and then mapped onto `BlsScalar` in the Bls12_381 scalar field.
#![allow(non_snake_case)]

use dusk_bls12_381::BlsScalar;
use lazy_static::lazy_static;

const CONSTANTS: usize = 960;

lazy_static! {
  /// `ROUND_CONSTANTS` constists on a static reference
  /// that points to the pre-loaded 960 Fq constants.
  ///
  /// This 960 `BlsScalar` constants are loaded from `ark.bin`
  /// where all of the `BlsScalar`s are represented in bytes.
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
    use dusk_bls12_381::BlsScalar;

    #[test]
    fn test_round_constants() {
        // Check each element is non-zero
        let zero = BlsScalar::zero();
        let has_zero = ROUND_CONSTANTS.iter().any(|&x| x == zero);
        for ctant in ROUND_CONSTANTS.iter() {
            let bytes = ctant.to_bytes();
            assert!(&BlsScalar::from_bytes(&bytes).unwrap() == ctant);
        }
        assert!(!has_zero);
    }
}
