#![allow(non_snake_case)]
use lazy_static::*;

use bulletproofs::r1cs::LinearCombination;
use curve25519_dalek::scalar::Scalar;
use std::ops::Mul;

/// This represents the ammount of inputs used to perform
/// a permutation (padding included).
/// 
/// So the number of inputs is `WIDTH - 1`.
const WIDTH: usize = 9;

lazy_static! {
  /// Represents a `static reference` to the 
  /// `Maximum Distance Separable Matrix -> MDS_MATRIX` 
  /// of `(WIDTH x WIDTH)`.
  /// 
  /// This matrix is loaded from the `mds.bin` file where 
  /// the matrix is pre-computed and represented in bytes.
  pub static ref MDS_MATRIX: [[Scalar; WIDTH]; WIDTH] = {
    let bytes = include_bytes!("mds.bin");

    assert_eq!(bytes.len(), (WIDTH * WIDTH) << 5);

    unsafe { std::ptr::read(bytes.as_ptr() as *const _) }
  };
}

/// Represents the product `row_vec * colum_vec` for the
/// `Scalar` use case. 
/// 
/// This operation returns a Scalar as a result.
fn dot_product(a: &[Scalar], b: &[Scalar]) -> Scalar {
  a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Represents the product `row_lc_vec * colum_scalar_vec` for the
/// `LinearCombination` use case. 
/// 
/// This operation returns a `simplified` `LinearCombination`
/// which means that the `LinearCombination` gets simplified by
/// taking Variables common across terms and adding their 
/// corresponding scalars. 
/// This, at the end, reduces the size of the vector that holds
/// the `lcs`.
fn dot_product_lc(a: &[Scalar], b: Vec<LinearCombination>) -> LinearCombination {
  let l_cs: Vec<LinearCombination> = a
    .iter()
    .zip(b.iter())
    .map(|(a_i, b_i)| *a_i * b_i.clone())
    .collect();

  let mut sum: LinearCombination = Scalar::zero().into();

  for l_c in l_cs {
    sum = sum + l_c;
  }

  sum.simplify()
}

impl<'a> Mul<&'a MDS_MATRIX> for Vec<Scalar> {
  type Output = Vec<Scalar>;
  /// Performs the `mul` between a Matrix of `Scalar` and 
  /// a vector of `Scalar` which results on a `Vec<Scalar>`.
  fn mul(self, rhs: &'a MDS_MATRIX) -> Vec<Scalar> {
    rhs.iter().map(|row| dot_product(row, &self)).collect()
  }
}

impl<'a> Mul<&'a MDS_MATRIX> for Vec<LinearCombination> {
  type Output = Vec<LinearCombination>;
  /// Performs the `mul` between a Matrix of `Scalar` and 
  /// a vector of `LinearCombination` which results on a 
  /// `Vec<LinearCombination>`.
  fn mul(self, rhs: &'a MDS_MATRIX) -> Vec<LinearCombination> {
    rhs
      .iter()
      .map(|row| dot_product_lc(row, self.clone()))
      .collect()
  }
}
