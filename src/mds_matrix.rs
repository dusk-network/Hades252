#![allow(non_snake_case)]
use lazy_static::*;

use crate::linear_combination::LinearCombination;
use crate::scalar::Scalar;
use crate::WIDTH;
use std::ops::Mul;

lazy_static! {
    pub static ref MDS_MATRIX: [[Scalar; WIDTH]; WIDTH] = {
        let bytes = include_bytes!("mds.bin");

        assert_eq!(bytes.len(), (WIDTH * WIDTH) << 5);

        unsafe { std::ptr::read(bytes.as_ptr() as *const _) }
    };
}

fn dot_product(a: &[Scalar], b: &[Scalar]) -> Scalar {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn dot_product_lc(a: &[Scalar], b: Vec<LinearCombination>) -> LinearCombination {
    let l_cs: Vec<LinearCombination> = a
        .iter()
        .zip(b.iter())
        .map(|(a_i, b_i)| a_i.clone() * b_i.clone())
        .collect();

    let mut sum: LinearCombination = Scalar::zero().into();

    for l_c in l_cs {
        sum = sum + l_c;
    }

    sum.simplify()
}

impl<'a> Mul<&'a MDS_MATRIX> for Vec<Scalar> {
    type Output = Vec<Scalar>;
    fn mul(self, rhs: &'a MDS_MATRIX) -> Vec<Scalar> {
        rhs.iter().map(|row| dot_product(row, &self)).collect()
    }
}

impl<'a> Mul<&'a MDS_MATRIX> for Vec<LinearCombination> {
    type Output = Vec<LinearCombination>;
    fn mul(self, rhs: &'a MDS_MATRIX) -> Vec<LinearCombination> {
        rhs.iter()
            .map(|row| dot_product_lc(row, self.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    // TODO Grant `MDS_MATRIX` holds all properties of a MDS matrix
}
