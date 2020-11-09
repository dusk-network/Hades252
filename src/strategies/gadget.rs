// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::Strategy;
use crate::mds_matrix::MDS_MATRIX;
use crate::WIDTH;
use dusk_bls12_381::BlsScalar;
use dusk_plonk::prelude::*;

/// Implements a Hades252 strategy for `Variable` as input values.
/// Requires a reference to a `ConstraintSystem`.
pub struct GadgetStrategy<'a> {
    /// A reference to the constraint system used by the gadgets
    pub cs: &'a mut StandardComposer,
    zero: Variable,
    count: usize,
}

impl<'a> GadgetStrategy<'a> {
    /// Constructs a new `GadgetStrategy` with the constraint system.
    pub fn new(cs: &'a mut StandardComposer) -> Self {
        let zero = cs.add_input(BlsScalar::zero());
        cs.constrain_to_constant(zero, BlsScalar::zero(), BlsScalar::zero());

        GadgetStrategy { cs, zero, count: 0 }
    }

    /// Perform the hades permutation on a plonk circuit
    pub fn hades_gadget(composer: &'a mut StandardComposer, x: &mut [Variable]) {
        let mut strategy = GadgetStrategy::new(composer);

        strategy.perm(x);
    }
}

impl<'a> Strategy<Variable> for GadgetStrategy<'a> {
    fn add_round_key<'b, I>(&mut self, constants: &mut I, words: &mut [Variable])
    where
        I: Iterator<Item = &'b BlsScalar>,
    {
        // Add only for the first round.
        //
        // The remainder ARC are performed with the constant appended
        // to the linear layer
        if self.count == 0 {
            words.iter_mut().for_each(|w| {
                *w = self.cs.add(
                    (BlsScalar::one(), *w),
                    (BlsScalar::zero(), self.zero),
                    Self::next_c(constants),
                    BlsScalar::zero(),
                );
            });
        }
    }

    fn quintic_s_box(&mut self, value: &mut Variable) {
        let v2 = self.cs.mul(
            BlsScalar::one(),
            *value,
            *value,
            BlsScalar::zero(),
            BlsScalar::zero(),
        );

        let v4 = self.cs.mul(
            BlsScalar::one(),
            v2,
            v2,
            BlsScalar::zero(),
            BlsScalar::zero(),
        );

        *value = self.cs.mul(
            BlsScalar::one(),
            v4,
            *value,
            BlsScalar::zero(),
            BlsScalar::zero(),
        );
    }

    /// Adds a constraint for each matrix coefficient multiplication
    fn mul_matrix<'b, I>(&mut self, constants: &mut I, values: &mut [Variable])
    where
        I: Iterator<Item = &'b BlsScalar>,
    {
        let mut result = [self.zero; WIDTH];
        self.count += 1;

        // Implementation optimized for WIDTH = 5
        //
        // c is the next round constant.
        // For the partial round, it is added only for the last element
        //
        // The resulting array `r` will be defined as
        // r[x] = sum j 0..WIDTH ( MDS[x][j] * values[j] ) + c
        //
        // q_l = MDS[x][0]
        // q_r = MDS[x][1]
        // q_4 = MDS[x][2]
        // w_l = values[0]
        // w_r = values[1]
        // w_4 = values[2]
        // r[x] = q_l · w_l + q_r · w_r + q_4 · w_4;
        //
        // q_l = MDS[x][3]
        // q_r = MDS[x][4]
        // q_4 = 1
        // w_l = values[3]
        // w_r = values[4]
        // w_4 = r[x]
        // r[x] = q_l · w_l + q_r · w_r + q_4 · w_4 + c;
        for j in 0..WIDTH {
            let c;

            if self.count < Self::rounds() {
                c = Self::next_c(constants);
            } else {
                c = BlsScalar::zero();
            }

            result[j] = self.cs.big_add(
                (MDS_MATRIX[j][0], values[0]),
                (MDS_MATRIX[j][1], values[1]),
                Some((MDS_MATRIX[j][2], values[2])),
                BlsScalar::zero(),
                BlsScalar::zero(),
            );

            result[j] = self.cs.big_add(
                (MDS_MATRIX[j][3], values[3]),
                (MDS_MATRIX[j][4], values[4]),
                Some((BlsScalar::one(), result[j])),
                c,
                BlsScalar::zero(),
            );
        }

        values.copy_from_slice(&result);
    }
}

#[cfg(test)]
mod tests {
    use crate::{GadgetStrategy, ScalarStrategy, Strategy, WIDTH};

    use anyhow::Result;
    use dusk_plonk::prelude::*;
    use std::mem;

    fn perm(values: &mut [BlsScalar]) {
        let mut strategy = ScalarStrategy::new();
        strategy.perm(values);
    }

    #[test]
    fn hades_preimage() -> Result<()> {
        const CAPACITY: usize = 2048;

        fn hades() -> ([BlsScalar; WIDTH], [BlsScalar; WIDTH]) {
            let mut input = [BlsScalar::zero(); WIDTH];
            input
                .iter_mut()
                .for_each(|s| *s = BlsScalar::random(&mut rand::thread_rng()));
            let mut output = [BlsScalar::zero(); WIDTH];
            output.copy_from_slice(&input);
            ScalarStrategy::new().perm(&mut output);
            (input, output)
        }

        fn hades_gadget_tester(
            i: [BlsScalar; WIDTH],
            o: [BlsScalar; WIDTH],
            composer: &mut StandardComposer,
        ) -> Vec<BlsScalar> {
            let mut perm: [Variable; WIDTH] = [unsafe { mem::zeroed() }; WIDTH];

            let zero = composer.add_input(BlsScalar::zero());

            let mut i_var: [Variable; WIDTH] = [zero; WIDTH];
            i.iter().zip(i_var.iter_mut()).for_each(|(i, v)| {
                *v = composer.add_input(*i);
            });

            let mut o_var: [Variable; WIDTH] = [zero; WIDTH];
            o.iter().zip(o_var.iter_mut()).for_each(|(o, v)| {
                *v = composer.add_input(*o);
            });

            // Apply Hades gadget strategy.
            GadgetStrategy::hades_gadget(composer, &mut i_var);

            // Copy the result of the permutation into the perm.
            perm.copy_from_slice(&i_var);

            // Check that the Gadget perm results = BlsScalar perm results
            i_var.iter().zip(o_var.iter()).for_each(|(p, o)| {
                composer.add_gate(
                    *p,
                    *o,
                    zero,
                    -BlsScalar::one(),
                    BlsScalar::one(),
                    BlsScalar::zero(),
                    BlsScalar::zero(),
                    BlsScalar::zero(),
                );
            });

            composer.add_dummy_constraints();
            vec![BlsScalar::zero()]
        }

        // Setup OG params.
        let public_parameters = PublicParameters::setup(CAPACITY, &mut rand::thread_rng())?;
        let (ck, vk) = public_parameters.trim(CAPACITY)?;

        let (i, o) = hades();
        // Proving
        let mut prover = Prover::new(b"hades_gadget_tester");
        let pi = hades_gadget_tester(i, o, prover.mut_cs());
        prover.preprocess(&ck)?;
        let proof = prover.prove(&ck)?;

        // Verifying
        let mut verifier = Verifier::new(b"hades_gadget_tester");
        let _ = hades_gadget_tester(i, o, verifier.mut_cs());
        verifier.preprocess(&ck)?;
        assert!(verifier.verify(&proof, &vk, &pi).is_ok());
        //------------------------------------------//
        //                                          //
        //  Second Proof test with different values //
        //                                          //
        //------------------------------------------//

        // Prepare input & output of the permutation for second Proof test
        prover.clear_witness();
        let e = [BlsScalar::from(5000u64); WIDTH];
        let mut e_perm = [BlsScalar::from(5000u64); WIDTH];
        perm(&mut e_perm);

        // Prove 2 with different values
        let pi2 = hades_gadget_tester(e, e_perm, prover.mut_cs());
        let proof2 = prover.prove(&ck)?;

        // Verify 2 with different values
        // Verifying
        let _ = hades_gadget_tester(i, o, verifier.mut_cs());
        assert!(verifier.verify(&proof2, &vk, &pi2).is_ok());

        //------------------------------------------//
        //                                          //
        //  Third Proof test with wrong values      //
        //                                          //
        //------------------------------------------//

        // Generate [31, 0, 0, 0, 0] as real input to the perm but build the
        // proof with [31, 31, 31, 31, 31]. This should fail on verification
        // since the Proof contains incorrect statements.
        let x_scalar = BlsScalar::from(31u64);
        let mut x = [BlsScalar::zero(); WIDTH];
        x[1] = x_scalar;
        let mut h = [BlsScalar::from(31u64); WIDTH];
        perm(&mut h);

        // Prove 3 with wrong inputs
        prover.clear_witness();
        let pi3 = hades_gadget_tester(x, h, prover.mut_cs());
        let proof3 = prover.prove(&ck)?;

        // Verify 3 with wrong inputs should fail
        let _ = hades_gadget_tester(i, o, verifier.mut_cs());
        assert!(verifier.verify(&proof3, &vk, &pi3).is_err());

        Ok(())
    }
}
