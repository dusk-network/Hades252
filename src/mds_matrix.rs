#![allow(non_snake_case)]
use crate::{Scalar, WIDTH};

use algebra::biginteger::BigInteger256;
use lazy_static::lazy_static;
use num_traits::Zero;

lazy_static! {
  /// Represents a `static reference` to the
  /// `Maximum Distance Separable Matrix -> MDS_MATRIX`
  /// of `(WIDTH x WIDTH)`.
  ///
  /// This matrix is loaded from the `mds.bin` file where
  /// is pre-computed and represented in bytes.
  pub static ref MDS_MATRIX: [[Scalar; WIDTH]; WIDTH] = {
      let bytes = include_bytes!("../assets/mds.bin");
      let mut mds = [[Scalar::zero(); WIDTH]; WIDTH];
      let mut k = 0;
      let mut a = [0x00u8; 8];
      let mut b = [0x00u8; 8];
      let mut c = [0x00u8; 8];
      let mut d = [0x00u8; 8];

      for i in 0..WIDTH {
          for j in 0..WIDTH {
              a.copy_from_slice(&bytes[k..k+8]);
              b.copy_from_slice(&bytes[k+8..k+16]);
              c.copy_from_slice(&bytes[k+16..k+24]);
              d.copy_from_slice(&bytes[k+24..k+32]);
              k += 32;

              mds[i][j] = Scalar::from(BigInteger256([
                      u64::from_le_bytes(a),
                      u64::from_le_bytes(b),
                      u64::from_le_bytes(c),
                      u64::from_le_bytes(d)])
                  );
          }
      }

      mds
  };
}
