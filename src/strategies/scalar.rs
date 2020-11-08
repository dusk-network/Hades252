// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::Strategy;
use crate::{mds_matrix::MDS_MATRIX, WIDTH};
use dusk_bls12_381::Scalar;

/// Implements a Hades252 strategy for `Scalar` as input values.
#[derive(Default)]
pub struct ScalarStrategy {}

impl ScalarStrategy {
    /// Constructs a new `ScalarStrategy`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Strategy<Scalar> for ScalarStrategy {
    fn add_round_key<'b, I>(&mut self, constants: &mut I, words: &mut [Scalar])
    where
        I: Iterator<Item = &'b Scalar>,
    {
        words.iter_mut().for_each(|w| {
            *w += Self::next_c(constants);
        });
    }

    fn quintic_s_box(&mut self, value: &mut Scalar) {
        *value = value.square().square() * *value;
    }

    fn mul_matrix<'b, I>(&mut self, _constants: &mut I, values: &mut [Scalar])
    where
        I: Iterator<Item = &'b Scalar>,
    {
        let mut result = [Scalar::zero(); WIDTH];

        for j in 0..WIDTH {
            for k in 0..WIDTH {
                result[k] += MDS_MATRIX[k][j] * values[j];
            }
        }

        values.copy_from_slice(&result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScalarStrategy, Strategy, WIDTH};

    fn perm(values: &mut [Scalar]) {
        let mut strategy = ScalarStrategy::new();
        strategy.perm(values);
    }

    #[test]
    fn hades_det() {
        let mut x = [Scalar::from(17u64); WIDTH];
        let mut y = [Scalar::from(17u64); WIDTH];
        let mut z = [Scalar::from(19u64); WIDTH];

        perm(&mut x);
        perm(&mut y);
        perm(&mut z);

        assert_eq!(x, y);
        assert_ne!(x, z);
    }
}
