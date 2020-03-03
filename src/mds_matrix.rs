#![allow(non_snake_case)]
use crate::{Scalar, WIDTH};

use lazy_static::lazy_static;
use num_traits::{One, Zero};

lazy_static! {
  /// Represents a `static reference` to the
  /// `Maximum Distance Separable Matrix -> MDS_MATRIX`
  /// of `(WIDTH x WIDTH)`.
  ///
  /// This matrix is loaded from the `mds.bin` file where
  /// is pre-computed and represented in bytes.
  pub static ref MDS_MATRIX: [[Scalar; WIDTH]; WIDTH] = mds();
}

fn mds() -> [[Scalar; WIDTH]; WIDTH] {
    let mut matrix = [[Scalar::zero(); WIDTH]; WIDTH];
    let mut xs = [Scalar::zero(); WIDTH];
    let mut ys = [Scalar::zero(); WIDTH];

    // Generate x and y values deterministically for the cauchy matrix
    // where x[i] != y[i] to allow the values to be inverted
    // and there are no duplicates in the x vector or y vector, so that the determinant is always non-zero
    // [a b]
    // [c d]
    // det(M) = (ad - bc) ; if a == b and c == d => det(M) =0
    // For an MDS matrix, every possible mxm submatrix, must have det(M) != 0
    (0..WIDTH).for_each(|i| {
        xs[i] = Scalar::from(i as u64);
        ys[i] = Scalar::from((i + WIDTH) as u64);
    });

    let mut m = 0;
    (0..WIDTH).for_each(|i| {
        (0..WIDTH).for_each(|j| {
            matrix[m][j] = Scalar::one() / (xs[i] + ys[j]);
        });
        m += 1;
    });

    matrix
}
