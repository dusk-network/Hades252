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

/// Implements a Hades252 strategy for `Witness` as input values.
/// Requires a reference to a `ConstraintSystem`.
pub struct GadgetStrategy<'a> {
    /// A reference to the constraint system used by the gadgets
    cs: &'a mut TurboComposer,
    count: usize,
}

impl<'a> GadgetStrategy<'a> {
    /// Constructs a new `GadgetStrategy` with the constraint system.
    pub fn new(cs: &'a mut TurboComposer) -> Self {
        GadgetStrategy { cs, count: 0 }
    }

    /// Perform the hades permutation on a plonk circuit
    pub fn gadget(composer: &'a mut TurboComposer, x: &mut [Witness]) {
        let mut strategy = GadgetStrategy::new(composer);

        strategy.perm(x);
    }
}

impl AsMut<TurboComposer> for GadgetStrategy<'_> {
    fn as_mut(&mut self) -> &mut TurboComposer {
        self.cs
    }
}

impl<'a> Strategy<Witness> for GadgetStrategy<'a> {
    fn add_round_key<'b, I>(&mut self, constants: &mut I, words: &mut [Witness])
    where
        I: Iterator<Item = &'b BlsScalar>,
    {
        // Add only for the first round.
        //
        // The remainder ARC are performed with the constant appended
        // to the linear layer
        if self.count == 0 {
            words.iter_mut().for_each(|w| {
                let constant = Self::next_c(constants);
                let constraint = Constraint::new().left(1).a(*w).constant(constant);

                *w = self.cs.gate_add(constraint);
            });
        }
    }

    fn quintic_s_box(&mut self, value: &mut Witness) {
        let constraint = Constraint::new().mult(1).a(*value).b(*value);
        let v2 = self.cs.gate_mul(constraint);

        let constraint = Constraint::new().mult(1).a(v2).b(v2);
        let v4 = self.cs.gate_mul(constraint);

        let constraint = Constraint::new().mult(1).a(v4).b(*value);
        *value = self.cs.gate_mul(constraint);
    }

    /// Adds a constraint for each matrix coefficient multiplication
    fn mul_matrix<'b, I>(&mut self, constants: &mut I, values: &mut [Witness])
    where
        I: Iterator<Item = &'b BlsScalar>,
    {
        let zero = TurboComposer::constant_zero();

        let mut result = [zero; WIDTH];
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

            let constraint = Constraint::new()
                .left(MDS_MATRIX[j][0])
                .a(values[0])
                .right(MDS_MATRIX[j][1])
                .b(values[1])
                .fourth(MDS_MATRIX[j][2])
                .d(values[2]);

            result[j] = self.cs.gate_add(constraint);

            let constraint = Constraint::new()
                .left(MDS_MATRIX[j][3])
                .a(values[3])
                .right(MDS_MATRIX[j][4])
                .b(values[4])
                .fourth(1)
                .d(result[j])
                .constant(c);

            result[j] = self.cs.gate_add(constraint);
        }

        values.copy_from_slice(&result);
    }
}

#[cfg(test)]
mod tests {
    use crate::{GadgetStrategy, ScalarStrategy, Strategy, WIDTH};
    use core::result::Result;
    use dusk_plonk::prelude::*;

    /// Generate a random input and perform a permutation
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

    /// Permutate `i` in a circuit and assert the result equals `o`
    fn hades_gadget_tester(
        i: [BlsScalar; WIDTH],
        o: [BlsScalar; WIDTH],
        composer: &mut TurboComposer,
    ) {
        let zero = TurboComposer::constant_zero();

        let mut perm: [Witness; WIDTH] = [zero; WIDTH];

        let mut i_var: [Witness; WIDTH] = [zero; WIDTH];
        i.iter().zip(i_var.iter_mut()).for_each(|(i, v)| {
            *v = composer.append_witness(*i);
        });

        let mut o_var: [Witness; WIDTH] = [zero; WIDTH];
        o.iter().zip(o_var.iter_mut()).for_each(|(o, v)| {
            *v = composer.append_witness(*o);
        });

        // Apply Hades gadget strategy.
        GadgetStrategy::gadget(composer, &mut i_var);

        // Copy the result of the permutation into the perm.
        perm.copy_from_slice(&i_var);

        // Check that the Gadget perm results = BlsScalar perm results
        i_var.iter().zip(o_var.iter()).for_each(|(p, o)| {
            composer.assert_equal(*p, *o);
        });
    }

    /// Setup the ZK parameters
    fn setup() -> Result<(&'static [u8], Verifier, CommitKey, OpeningKey), Error> {
        const CAPACITY: usize = 1 << 10;

        let pp = PublicParameters::setup(CAPACITY, &mut rand::thread_rng())?;
        let label = b"hades_gadget_tester";
        let (ck, ok) = pp.trim(CAPACITY)?;

        let (i, o) = hades();

        let mut prover = Prover::new(label);
        hades_gadget_tester(i, o, prover.composer_mut());

        let mut verifier = Verifier::new(label);

        hades_gadget_tester(i, o, verifier.composer_mut());
        verifier.preprocess(&ck)?;

        Ok((label, verifier, ck, ok))
    }

    #[test]
    fn preimage() -> Result<(), Error> {
        let (label, verifier, ck, ok) = setup()?;

        let (i, o) = hades();

        // Proving
        let mut prover = Prover::new(label);
        hades_gadget_tester(i, o, prover.composer_mut());
        let proof = prover.prove(&ck)?;

        // Verifying
        verifier.verify(&proof, &ok, &[])?;

        Ok(())
    }

    #[test]
    fn preimage_constant() -> Result<(), Error> {
        let (label, verifier, ck, ok) = setup()?;

        // Prepare input & output
        let e = [BlsScalar::from(5000u64); WIDTH];
        let mut e_perm = [BlsScalar::from(5000u64); WIDTH];
        ScalarStrategy::new().perm(&mut e_perm);

        // Proving
        let mut prover = Prover::new(label);
        hades_gadget_tester(e, e_perm, prover.composer_mut());
        let proof = prover.prove(&ck)?;

        // Verifying
        verifier.verify(&proof, &ok, &[])?;

        Ok(())
    }

    #[test]
    fn preimage_fails() -> Result<(), Error> {
        let (label, verifier, ck, ok) = setup()?;

        // Generate [31, 0, 0, 0, 0] as real input to the perm but build the
        // proof with [31, 31, 31, 31, 31]. This should fail on verification
        // since the Proof contains incorrect statements.
        let x_scalar = BlsScalar::from(31u64);

        let mut x = [BlsScalar::zero(); WIDTH];
        x[1] = x_scalar;

        let mut h = [BlsScalar::from(31u64); WIDTH];
        ScalarStrategy::new().perm(&mut h);

        // Prove 3 with wrong inputs
        let mut prover = Prover::new(label);
        hades_gadget_tester(x, h, prover.composer_mut());
        let proof = prover.prove(&ck)?;

        // Verification fails
        assert!(verifier.verify(&proof, &ok, &[]).is_err());

        Ok(())
    }
}
