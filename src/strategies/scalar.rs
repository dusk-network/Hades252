// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.”
use super::Strategy;
use crate::{mds_matrix::MDS_MATRIX, WIDTH};
use dusk_plonk::bls12_381::Scalar as BlsScalar;

/// Implements a Hades252 strategy for `BlsScalar` as input values.
#[derive(Default)]
pub struct ScalarStrategy {}

impl ScalarStrategy {
    /// Constructs a new `ScalarStrategy`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Strategy<BlsScalar> for ScalarStrategy {
    fn quintic_s_box(&mut self, value: &mut BlsScalar) {
        // XXX: Check speed difference between this and pow fn
        let s = *value;
        *value = s * s * s * s * s;
    }

    fn mul_matrix(&mut self, values: &mut [BlsScalar]) {
        let mut result = [BlsScalar::zero(); WIDTH];

        for j in 0..WIDTH {
            for k in 0..WIDTH {
                result[k] += MDS_MATRIX[k][j] * values[j];
            }
        }

        values.copy_from_slice(&result);
    }

    fn add_round_key<'b, I>(&mut self, constants: &mut I, words: &mut [BlsScalar])
    where
        I: Iterator<Item = &'b BlsScalar>,
    {
        words.iter_mut().for_each(|w| {
            // XXX: Shouldn't it follow the impl of gadget returning err if we get out of constants?
            *w += constants.next().unwrap_or(&BlsScalar::one());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScalarStrategy, Strategy, WIDTH};

    fn perm(values: &mut [BlsScalar]) {
        let mut strategy = ScalarStrategy::new();
        strategy.perm(values);
    }

    #[test]
    fn hades_det() {
        let mut x = [BlsScalar::from(17u64); WIDTH];
        let mut y = [BlsScalar::from(17u64); WIDTH];
        let mut z = [BlsScalar::from(19u64); WIDTH];

        perm(&mut x);
        perm(&mut y);
        perm(&mut z);

        assert_eq!(x, y);
        assert_ne!(x, z);
    }
}
