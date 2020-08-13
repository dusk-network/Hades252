use super::Strategy;
use crate::{mds_matrix::MDS_MATRIX, WIDTH};
use dusk_plonk::prelude::*;

#[cfg(feature = "trace")]
use tracing::trace;

/// Size of the generated public inputs for the permutation gadget
pub const PI_SIZE: usize = 1737;

/// Implements a Hades252 strategy for `Variable` as input values.
/// Requires a reference to a `ConstraintSystem`.
pub struct GadgetStrategy<'a> {
    /// A reference to the constraint system used by the gadgets
    pub cs: &'a mut StandardComposer,
}

impl<'a> GadgetStrategy<'a> {
    /// Constructs a new `GadgetStrategy` with the constraint system.
    pub fn new(cs: &'a mut StandardComposer) -> Self {
        GadgetStrategy { cs }
    }

    /// Perform the hades permutation on a plonk circuit
    pub fn hades_gadget(composer: &'a mut StandardComposer, x: &mut [Variable]) {
        #[cfg(feature = "trace")]
        let circuit_size = composer.circuit_size();

        let mut strategy = GadgetStrategy::new(composer);

        strategy.perm(x);

        #[cfg(feature = "trace")]
        {
            trace!(
                "Hades permutation performed with {} constraints for {} bits",
                strategy.cs.circuit_size() - circuit_size,
                WIDTH
            );
        }
    }
}

impl<'a> Strategy<Variable> for GadgetStrategy<'a> {
    fn quintic_s_box(&mut self, value: &mut Variable) {
        #[cfg(feature = "trace")]
        let circuit_size = self.cs.circuit_size();

        let v = value.clone();

        (0..2).for_each(|_| {
            *value = self.cs.mul(
                BlsScalar::one(),
                *value,
                *value,
                BlsScalar::zero(),
                BlsScalar::zero(),
            )
        });

        *value = self.cs.mul(
            BlsScalar::one(),
            *value,
            v,
            BlsScalar::zero(),
            BlsScalar::zero(),
        );

        #[cfg(feature = "trace")]
        {
            trace!(
                "Hades quintic S-Box performed with {} constraints for {} bits",
                self.cs.circuit_size() - circuit_size,
                WIDTH
            );
        }
    }

    /// Adds a constraint for each matrix coefficient multiplication
    fn mul_matrix(&mut self, values: &mut [Variable]) {
        #[cfg(feature = "trace")]
        let circuit_size = self.cs.circuit_size();

        // Declare and constraint zero.
        let zero = self.cs.add_input(BlsScalar::zero());
        self.cs
            .constrain_to_constant(zero, BlsScalar::zero(), BlsScalar::zero());

        let mut product = [zero; WIDTH];
        let mut z3 = zero;

        for j in 0..WIDTH {
            for k in 0..WIDTH / 4 {
                let i = 4 * k;

                let z1 = self.cs.add(
                    (MDS_MATRIX[j][i], values[i]),
                    (MDS_MATRIX[j][i + 1], values[i + 1]),
                    BlsScalar::zero(),
                    BlsScalar::zero(),
                );

                let z2 = self.cs.add(
                    (MDS_MATRIX[j][i + 2], values[i + 2]),
                    (MDS_MATRIX[j][i + 3], values[i + 3]),
                    BlsScalar::zero(),
                    BlsScalar::zero(),
                );

                z3 = self.cs.add(
                    (BlsScalar::one(), z1),
                    (BlsScalar::one(), z2),
                    BlsScalar::zero(),
                    BlsScalar::zero(),
                );
            }

            // TODO - Replace by compiler constant evaluation
            if WIDTH < 4 {
                for k in 0..WIDTH {
                    product[k] = self.cs.add(
                        (BlsScalar::one(), product[k]),
                        (MDS_MATRIX[k][j], values[j]),
                        BlsScalar::zero(),
                        BlsScalar::zero(),
                    );
                }
            } else if WIDTH & 1 == 1 {
                product[j] = self.cs.add(
                    (BlsScalar::one(), z3),
                    (MDS_MATRIX[j][WIDTH - 1], values[WIDTH - 1]),
                    BlsScalar::zero(),
                    BlsScalar::zero(),
                );
            } else {
                product[j] = z3;
            }
        }

        values.copy_from_slice(&product);

        #[cfg(feature = "trace")]
        {
            trace!(
                "Hades MDS multiplication performed with {} constraints for {} bits",
                self.cs.circuit_size() - circuit_size,
                WIDTH
            );
        }
    }

    /// Multiply the values for MDS matrix in the partial round application process.
    fn mul_matrix_partial_round(&mut self, constants: &[BlsScalar], values: &mut [Variable]) {
        #[cfg(feature = "trace")]
        let circuit_size = self.cs.circuit_size();

        // Declare and constraint zero.
        let zero = self.cs.add_input(BlsScalar::zero());
        self.cs
            .constrain_to_constant(zero, BlsScalar::zero(), BlsScalar::zero());

        let mut product = [zero; WIDTH];
        let mut z3 = zero;

        for j in 0..WIDTH {
            for k in 0..WIDTH / 4 {
                let i = 4 * k;

                let q_c = constants[i] * MDS_MATRIX[j][i] + constants[i + 1] * MDS_MATRIX[j][i + 1];
                let z1 = self.cs.add(
                    (MDS_MATRIX[j][i], values[i]),
                    (MDS_MATRIX[j][i + 1], values[i + 1]),
                    q_c,
                    BlsScalar::zero(),
                );

                let q_c = constants[i + 2] * MDS_MATRIX[j][i + 2]
                    + constants[i + 3] * MDS_MATRIX[j][i + 3];
                let z2 = self.cs.add(
                    (MDS_MATRIX[j][i + 2], values[i + 2]),
                    (MDS_MATRIX[j][i + 3], values[i + 3]),
                    q_c,
                    BlsScalar::zero(),
                );

                z3 = self.cs.add(
                    (BlsScalar::one(), z1),
                    (BlsScalar::one(), z2),
                    BlsScalar::zero(),
                    BlsScalar::zero(),
                );
            }

            // TODO - Replace by compiler constant evaluation
            if WIDTH < 4 {
                for k in 0..WIDTH {
                    product[k] = self.cs.add(
                        (BlsScalar::one(), product[k]),
                        (MDS_MATRIX[k][j], values[j]),
                        constants[j] * MDS_MATRIX[k][j],
                        BlsScalar::zero(),
                    );
                }
            } else if WIDTH & 1 == 1 {
                product[j] = self.cs.add(
                    (BlsScalar::one(), z3),
                    (MDS_MATRIX[j][WIDTH - 1], values[WIDTH - 1]),
                    constants[WIDTH - 1] * MDS_MATRIX[j][WIDTH - 1],
                    BlsScalar::zero(),
                );
            } else {
                product[j] = z3;
            }
        }

        values.copy_from_slice(&product);

        #[cfg(feature = "trace")]
        {
            trace!(
                "Hades MDS multiplication performed with {} constraints for {} bits",
                self.cs.circuit_size() - circuit_size,
                WIDTH
            );
        }
    }

    fn add_round_key<'b, I>(&mut self, constants: &mut I, words: &mut [Variable])
    where
        I: Iterator<Item = &'b BlsScalar>,
    {
        #[cfg(feature = "trace")]
        let circuit_size = self.cs.circuit_size();

        // Declare and constraint zero.
        let zero = self.cs.add_input(BlsScalar::zero());
        self.cs
            .constrain_to_constant(zero, BlsScalar::zero(), BlsScalar::zero());

        words.iter_mut().for_each(|w| {
            let p = constants
                .next()
                .cloned()
                .expect("Hades252 out of ARK constants");

            *w = self.cs.add(
                (BlsScalar::one(), *w),
                (BlsScalar::zero(), zero),
                p,
                BlsScalar::zero(),
            );
        });

        #[cfg(feature = "trace")]
        {
            trace!(
                "Hades ARK performed with {} constraints for {} bits",
                self.cs.circuit_size() - circuit_size,
                WIDTH
            );
        }
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

            // Check that the Gadget perm results = Scalar perm results
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
