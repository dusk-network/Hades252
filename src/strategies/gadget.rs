use super::Strategy;
use crate::{mds_matrix::MDS_MATRIX, BlsScalar, WIDTH};
use dusk_plonk::constraint_system::composer::StandardComposer;
use dusk_plonk::constraint_system::variable::Variable;

#[cfg(feature = "trace")]
use tracing::trace;

/// Size of the generated public inputs for the permutation gadget
pub const PI_SIZE: usize = 1737;

/// Implements a Hades252 strategy for `Variable` as input values.
/// Requires a reference to a `ConstraintSystem`.
pub struct GadgetStrategy<'a, P>
where
    P: Iterator<Item = &'a mut BlsScalar>,
{
    /// Counter over pushed PI
    pub pi_len: usize,
    /// A reference to the constraint system used by the gadgets
    pub cs: StandardComposer,
    /// Mutable iterator over the public inputs
    pub pi_iter: P,
    /// Variable representing zero
    pub zero: Variable,
}

impl<'a, P> GadgetStrategy<'a, P>
where
    P: Iterator<Item = &'a mut BlsScalar>,
{
    /// Constructs a new `GadgetStrategy` with the constraint system.
    pub fn new(mut cs: StandardComposer, pi_iter: P) -> Self {
        let zero = cs.add_input(BlsScalar::zero());
        GadgetStrategy {
            pi_len: 0,
            cs,
            pi_iter,
            zero,
        }
    }

    /// Return the inner iterator over public inputs
    pub fn into_inner(self) -> (StandardComposer, P) {
        (self.cs, self.pi_iter)
    }

    /// Perform the hades permutation on a plonk circuit
    pub fn hades_gadget(
        composer: StandardComposer,
        pi: P,
        x: &mut [Variable],
    ) -> (StandardComposer, P) {
        #[cfg(feature = "trace")]
        let circuit_size = composer.circuit_size();

        let mut strategy = GadgetStrategy::new(composer, pi);

        strategy.perm(x);

        #[cfg(feature = "trace")]
        {
            trace!(
                "Hades permutation performed with {} constraints for {} bits",
                strategy.cs.circuit_size() - circuit_size,
                WIDTH
            );
        }

        strategy.into_inner()
    }

    /// Perform the poseidon hash on a plonk circuit
    pub fn poseidon_gadget(
        composer: StandardComposer,
        pi: P,
        x: &mut [Variable],
    ) -> (StandardComposer, P, Variable) {
        let (composer, pi) = GadgetStrategy::hades_gadget(composer, pi, x);
        (composer, pi, x[1])
    }

    /// Perform the poseidon slice hash on a plonk circuit
    pub fn poseidon_slice_gadget(
        composer: StandardComposer,
        pi: P,
        x: &[Variable],
    ) -> (StandardComposer, P, Variable) {
        let mut strategy = GadgetStrategy::new(composer, pi);

        let x = strategy.poseidon_slice(x);

        let (composer, pi) = strategy.into_inner();

        (composer, pi, x)
    }

    fn push_pi(&mut self, p: BlsScalar) {
        self.pi_len += 1;
        self.pi_iter
            .next()
            .map(|s| *s = p)
            .expect("Public inputs iterator depleted");
    }
}

impl<'a, P> Strategy<Variable> for GadgetStrategy<'a, P>
where
    P: Iterator<Item = &'a mut BlsScalar>,
{
    fn quintic_s_box(&mut self, value: &mut Variable) {
        #[cfg(feature = "trace")]
        let circuit_size = self.cs.circuit_size();

        let v = value.clone();

        (0..2).for_each(|_| {
            self.push_pi(BlsScalar::zero());
            *value = self.cs.mul(
                BlsScalar::one(),
                *value,
                *value,
                BlsScalar::zero(),
                BlsScalar::zero(),
            )
        });
        //println!("quintic sbox1 at gate: {}", self.cs.circuit_size());

        self.push_pi(BlsScalar::zero());
        *value = self.cs.mul(
            BlsScalar::one(),
            *value,
            v,
            BlsScalar::zero(),
            BlsScalar::zero(),
        );
        //println!("quintic sbox2 at gate: {}", self.cs.circuit_size());

        #[cfg(feature = "trace")]
        {
            trace!(
                "Hades quintic S-Box performed with {} constraints for {} bits",
                self.cs.circuit_size() - circuit_size,
                WIDTH
            );
        }
    }

    fn mul_matrix(&mut self, values: &mut [Variable]) {
        #[cfg(feature = "trace")]
        let circuit_size = self.cs.circuit_size();

        let mut product = [self.zero; WIDTH];
        let mut z3 = self.zero;

        for j in 0..WIDTH {
            for k in 0..WIDTH / 4 {
                let i = 4 * k;

                self.push_pi(BlsScalar::zero());
                let z1 = self.cs.add(
                    (MDS_MATRIX[j][i], values[i]),
                    (MDS_MATRIX[j][i + 1], values[i + 1]),
                    BlsScalar::zero(),
                    BlsScalar::zero(),
                );

                self.push_pi(BlsScalar::zero());
                let z2 = self.cs.add(
                    (MDS_MATRIX[j][i + 2], values[i + 2]),
                    (MDS_MATRIX[j][i + 3], values[i + 3]),
                    BlsScalar::zero(),
                    BlsScalar::zero(),
                );

                self.push_pi(BlsScalar::zero());
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
                    self.push_pi(BlsScalar::zero());
                    product[k] = self.cs.add(
                        (BlsScalar::one(), product[k]),
                        (MDS_MATRIX[k][j], values[j]),
                        BlsScalar::zero(),
                        BlsScalar::zero(),
                    );
                }
            } else if WIDTH & 1 == 1 {
                self.push_pi(BlsScalar::zero());
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

    /// Multiply the values for MDS matrix.
    fn mul_matrix_partial_round(&mut self, constants: &[BlsScalar], values: &mut [Variable]) {
        #[cfg(feature = "trace")]
        let circuit_size = self.cs.circuit_size();

        let mut product = [self.zero; WIDTH];
        let mut z3 = self.zero;

        for j in 0..WIDTH {
            for k in 0..WIDTH / 4 {
                let i = 4 * k;

                let q_c = constants[i] * MDS_MATRIX[j][i] + constants[i + 1] * MDS_MATRIX[j][i + 1];
                self.push_pi(BlsScalar::zero());
                let z1 = self.cs.add(
                    (MDS_MATRIX[j][i], values[i]),
                    (MDS_MATRIX[j][i + 1], values[i + 1]),
                    q_c,
                    BlsScalar::zero(),
                );

                let q_c = constants[i + 2] * MDS_MATRIX[j][i + 2]
                    + constants[i + 3] * MDS_MATRIX[j][i + 3];
                self.push_pi(BlsScalar::zero());
                let z2 = self.cs.add(
                    (MDS_MATRIX[j][i + 2], values[i + 2]),
                    (MDS_MATRIX[j][i + 3], values[i + 3]),
                    q_c,
                    BlsScalar::zero(),
                );

                self.push_pi(BlsScalar::zero());
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
                    self.push_pi(BlsScalar::zero());
                    product[k] = self.cs.add(
                        (BlsScalar::one(), product[k]),
                        (MDS_MATRIX[k][j], values[j]),
                        constants[j] * MDS_MATRIX[k][j],
                        BlsScalar::zero(),
                    );
                }
            } else if WIDTH & 1 == 1 {
                self.push_pi(BlsScalar::zero());
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

        words.iter_mut().for_each(|w| {
            let p = constants
                .next()
                .cloned()
                .expect("Hades252 out of ARK constants");

            self.push_pi(BlsScalar::zero());
            *w = self.cs.add(
                (BlsScalar::one(), *w),
                (BlsScalar::zero(), self.zero),
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

    /// Perform a slice strategy
    fn poseidon_slice(&mut self, data: &[Variable]) -> Variable {
        let mut perm = [self.zero; WIDTH];

        let mut elements = [self.zero; WIDTH - 2];
        elements
            .iter_mut()
            .enumerate()
            .for_each(|(i, e)| *e = self.cs.add_input(BlsScalar::from((i + 1) as u64)));

        data.chunks(WIDTH - 2).fold(self.zero, |r, chunk| {
            perm[0] = elements[chunk.len() - 1];
            perm[1] = r;

            chunk
                .iter()
                .zip(perm.iter_mut().skip(2))
                .for_each(|(c, p)| *p = *c);

            self.poseidon(&mut perm)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{BlsScalar, GadgetStrategy, ScalarStrategy, Strategy, WIDTH};

    use std::mem;

    use dusk_plonk::commitment_scheme::kzg10::PublicParameters;
    use dusk_plonk::constraint_system::variable::Variable;
    use dusk_plonk::constraint_system::StandardComposer;
    use dusk_plonk::fft::EvaluationDomain;
    use merlin::Transcript;

    fn perm(values: &mut [BlsScalar]) {
        let mut strategy = ScalarStrategy::new();
        strategy.perm(values);
    }

    fn gen_transcript() -> Transcript {
        Transcript::new(b"hades-plonk")
    }

    #[test]
    fn hades_preimage() {
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

        fn new_composer(
            i: [BlsScalar; WIDTH],
            o: [BlsScalar; WIDTH],
        ) -> (StandardComposer, Vec<BlsScalar>) {
            let mut perm: [Variable; WIDTH] = [unsafe { mem::zeroed() }; WIDTH];
            let mut composer = StandardComposer::new();
            let mut pi = vec![BlsScalar::zero(); CAPACITY];

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
            let (mut composer, _) =
                GadgetStrategy::hades_gadget(composer, pi.iter_mut(), &mut i_var);

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
            (composer, pi)
        }

        // Setup OG params.
        let public_parameters = PublicParameters::setup(CAPACITY, &mut rand::thread_rng()).unwrap();
        let (ck, vk) = public_parameters.trim(CAPACITY).unwrap();
        let domain = EvaluationDomain::new(CAPACITY).unwrap();

        let (i, o) = hades();
        let (mut composer, pi) = new_composer(i, o);
        let mut transcript = gen_transcript();
        // Preprocess circuit
        let circuit = composer.preprocess(&ck, &mut transcript, &domain);

        // Prove
        let proof = composer.prove(&ck, &circuit, &mut transcript.clone());

        // Verify
        assert!(proof.verify(&circuit, &mut transcript.clone(), &vk, pi.as_slice()));

        //------------------------------------------//
        //                                          //
        //  Second Proof test with different values //
        //                                          //
        //------------------------------------------//

        // Prepare input & output of the permutation for second Proof test
        let e = [BlsScalar::from(5000u64); WIDTH];
        let mut e_perm = [BlsScalar::from(5000u64); WIDTH];
        perm(&mut e_perm);

        // Prove 2 with different values
        let (mut composer_2, pi2) = new_composer(e, e_perm);
        let proof2 = composer_2.prove(&ck, &circuit, &mut transcript.clone());

        // Verify 2 with different values
        assert!(proof2.verify(&circuit, &mut transcript.clone(), &vk, pi2.as_slice()));

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
        let (mut composer_3, pi3) = new_composer(x, h);
        let proof3 = composer_3.prove(&ck, &circuit, &mut transcript.clone());

        // Verify 3 with wrong inputs should fail
        assert!(!proof3.verify(&circuit, &mut transcript.clone(), &vk, pi3.as_slice()));
    }

    #[test]
    fn poseidon_slice() {
        let public_parameters =
            PublicParameters::setup(4096 * 32, &mut rand::thread_rng()).unwrap();
        let (ck, vk) = PublicParameters::trim(&public_parameters, 4096 * 32).unwrap();
        let domain: EvaluationDomain = EvaluationDomain::new(4096 * 32).unwrap();

        // Generate circuit
        let mut base_transcript = gen_transcript();
        let mut composer: StandardComposer = StandardComposer::new();

        const BITS: usize = WIDTH * 20 - 19;
        const SLICE_PI_SIZE: usize = super::PI_SIZE * (1 + BITS / 2);

        let mut pi = vec![BlsScalar::zero(); SLICE_PI_SIZE];

        let data: Vec<BlsScalar> = (0..BITS)
            .map(|_| BlsScalar::random(&mut rand::thread_rng()))
            .collect();
        let result = ScalarStrategy::new().poseidon_slice(data.as_slice());
        let result = composer.add_input(result);

        let vars: Vec<Variable> = data.iter().map(|d| composer.add_input(*d)).collect();
        let (mut composer, _, x) =
            GadgetStrategy::poseidon_slice_gadget(composer, pi.iter_mut(), &vars);

        let zero = composer.add_input(BlsScalar::zero());
        composer.add_gate(
            result,
            x,
            zero,
            -BlsScalar::one(),
            BlsScalar::one(),
            BlsScalar::one(),
            BlsScalar::zero(),
            BlsScalar::zero(),
        );

        composer.add_dummy_constraints();

        let preprocessed_circuit = composer.preprocess(&ck, &mut base_transcript, &domain);

        // Prove
        let mut transcript = gen_transcript();
        let mut composer: StandardComposer = StandardComposer::new();

        let mut pi = vec![BlsScalar::zero(); SLICE_PI_SIZE];

        let data: Vec<BlsScalar> = (0..BITS)
            .map(|_| BlsScalar::random(&mut rand::thread_rng()))
            .collect();
        let result = ScalarStrategy::new().poseidon_slice(data.as_slice());
        let result = composer.add_input(result);

        let vars: Vec<Variable> = data.iter().map(|d| composer.add_input(*d)).collect();
        let (mut composer, _, x) =
            GadgetStrategy::poseidon_slice_gadget(composer, pi.iter_mut(), &vars);

        let zero = composer.add_input(BlsScalar::zero());
        composer.add_gate(
            result,
            x,
            zero,
            -BlsScalar::one(),
            BlsScalar::one(),
            BlsScalar::one(),
            BlsScalar::zero(),
            BlsScalar::zero(),
        );

        composer.add_dummy_constraints();

        let circuit = composer.preprocess(&ck, &mut transcript, &domain);
        let proof = composer.prove(&ck, &circuit, &mut transcript);

        // Verify
        let mut transcript = base_transcript.clone();
        assert!(proof.verify(&preprocessed_circuit, &mut transcript, &vk, &pi));
    }
}
