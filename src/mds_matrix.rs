#![allow(non_snake_case)]
use crate::{Scalar, WIDTH};

use lazy_static::lazy_static;

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
      let mut a = [0x00u8; 32];

      for i in 0..WIDTH {
          for j in 0..WIDTH {
              let k = i * WIDTH + j;
              a.copy_from_slice(&bytes[k..k+32]);
              let b = unsafe { std::mem::transmute_copy(&a) };
              mds[i][j] = Scalar::from_raw(b);
          }
      }

      mds
  };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mds() {
        // Check each element is non-zero
        let zero = Scalar::zero();
        let has_zero = MDS_MATRIX.iter().any(|row| row.iter().any(|&x| x == zero));
        assert!(!has_zero);
    }
}
