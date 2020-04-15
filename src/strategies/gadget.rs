use super::Strategy;
use crate::{mds_matrix::MDS_MATRIX, Bls12_381, Fq, WIDTH};

use num_traits::{One, Zero};
use plonk::cs::{composer::StandardComposer, constraint_system::Variable};

#[cfg(feature = "trace")]
use {plonk::cs::Composer, tracing::trace};

/// Size of the generated public inputs for the permutation gadget
pub const PI_SIZE: usize = 1736;

/// Implements a Hades252 strategy for `Variable` as input values.
/// Requires a reference to a `ConstraintSystem`.
pub struct GadgetStrategy<'a, P>
where
    P: Iterator<Item = &'a mut Fq>,
{
    /// Counter over pushed PI
    pub pi_len: usize,
    /// A reference to the constraint system used by the gadgets
    pub cs: StandardComposer<Bls12_381>,
    /// Mutable iterator over the public inputs
    pub pi_iter: P,
    /// Variable representing zero
    pub zero: Variable,
}

impl<'a, P> GadgetStrategy<'a, P>
where
    P: Iterator<Item = &'a mut Fq>,
{
    /// Constructs a new `GadgetStrategy` with the constraint system.
    pub fn new(mut cs: StandardComposer<Bls12_381>, pi_iter: P) -> Self {
        let zero = cs.add_input(Fq::zero());
        GadgetStrategy {
            pi_len: 0,
            cs,
            pi_iter,
            zero,
        }
    }

    /// Return the inner iterator over public inputs
    pub fn into_inner(self) -> (StandardComposer<Bls12_381>, P) {
        (self.cs, self.pi_iter)
    }

    /// Perform the hades permutation on a plonk circuit
    pub fn hades_gadget(
        composer: StandardComposer<Bls12_381>,
        pi: P,
        x: &mut [Variable],
    ) -> (StandardComposer<Bls12_381>, P) {
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
        composer: StandardComposer<Bls12_381>,
        pi: P,
        x: &mut [Variable],
    ) -> (StandardComposer<Bls12_381>, P, Variable) {
        let (composer, pi) = GadgetStrategy::hades_gadget(composer, pi, x);
        (composer, pi, x[1])
    }

    /// Perform the poseidon slice hash on a plonk circuit
    pub fn poseidon_slice_gadget(
        composer: StandardComposer<Bls12_381>,
        pi: P,
        x: &[Variable],
    ) -> (StandardComposer<Bls12_381>, P, Variable) {
        let mut strategy = GadgetStrategy::new(composer, pi);

        let x = strategy.poseidon_slice(x);

        let (composer, pi) = strategy.into_inner();

        (composer, pi, x)
    }

    /// Constrain x == h, being h a public input
    pub fn constrain_gadget(
        mut composer: StandardComposer<Bls12_381>,
        mut pi: P,
        x: &[Variable],
        h: &[Fq],
    ) -> (StandardComposer<Bls12_381>, P) {
        let zero = composer.add_input(Fq::zero());

        x.iter().zip(h.iter()).for_each(|(x, h)| {
            pi.next()
                .map(|s| *s = *h)
                .expect("Public inputs iterator depleted");

            composer.add_gate(
                *x,
                zero,
                zero,
                -Fq::one(),
                Fq::one(),
                Fq::one(),
                Fq::zero(),
                *h,
            );
        });

        (composer, pi)
    }

    fn push_pi(&mut self, p: Fq) {
        self.pi_len += 1;
        self.pi_iter
            .next()
            .map(|s| *s = p)
            .expect("Public inputs iterator depleted");
    }
}

impl<'a, P> Strategy<Variable> for GadgetStrategy<'a, P>
where
    P: Iterator<Item = &'a mut Fq>,
{
    fn quintic_s_box(&mut self, value: &mut Variable) {
        #[cfg(feature = "trace")]
        let circuit_size = self.cs.circuit_size();

        let v = *value;

        (0..2).for_each(|_| {
            self.push_pi(Fq::zero());
            *value = self.cs.mul(
                *value,
                *value,
                Fq::one(),
                -Fq::one(),
                Fq::zero(),
                Fq::zero(),
            )
        });

        self.push_pi(Fq::zero());
        *value = self
            .cs
            .mul(*value, v, Fq::one(), -Fq::one(), Fq::zero(), Fq::zero());

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

                self.push_pi(Fq::zero());
                let z1 = self.cs.add(
                    values[i],
                    values[i + 1],
                    MDS_MATRIX[j][i],
                    MDS_MATRIX[j][i + 1],
                    -Fq::one(),
                    Fq::zero(),
                    Fq::zero(),
                );

                self.push_pi(Fq::zero());
                let z2 = self.cs.add(
                    values[i + 2],
                    values[i + 3],
                    MDS_MATRIX[j][i + 2],
                    MDS_MATRIX[j][i + 3],
                    -Fq::one(),
                    Fq::zero(),
                    Fq::zero(),
                );

                self.push_pi(Fq::zero());
                z3 = self.cs.add(
                    z1,
                    z2,
                    Fq::one(),
                    Fq::one(),
                    -Fq::one(),
                    Fq::zero(),
                    Fq::zero(),
                );
            }

            // TODO - Replace by compiler constant evaluation
            if WIDTH < 4 {
                for k in 0..WIDTH {
                    self.push_pi(Fq::zero());
                    product[k] = self.cs.add(
                        product[k],
                        values[j],
                        Fq::one(),
                        MDS_MATRIX[k][j],
                        -Fq::one(),
                        Fq::zero(),
                        Fq::zero(),
                    );
                }
            } else if WIDTH & 1 == 1 {
                self.push_pi(Fq::zero());
                product[j] = self.cs.add(
                    z3,
                    values[WIDTH - 1],
                    Fq::one(),
                    MDS_MATRIX[j][WIDTH - 1],
                    -Fq::one(),
                    Fq::zero(),
                    Fq::zero(),
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
    fn mul_matrix_partial_round(&mut self, constants: &[Fq], values: &mut [Variable]) {
        #[cfg(feature = "trace")]
        let circuit_size = self.cs.circuit_size();

        let mut product = [self.zero; WIDTH];
        let mut z3 = self.zero;

        for j in 0..WIDTH {
            for k in 0..WIDTH / 4 {
                let i = 4 * k;

                let q_c = constants[i] * MDS_MATRIX[j][i] + constants[i + 1] * MDS_MATRIX[j][i + 1];
                self.push_pi(Fq::zero());
                let z1 = self.cs.add(
                    values[i],
                    values[i + 1],
                    MDS_MATRIX[j][i],
                    MDS_MATRIX[j][i + 1],
                    -Fq::one(),
                    q_c,
                    Fq::zero(),
                );

                let q_c = constants[i + 2] * MDS_MATRIX[j][i + 2]
                    + constants[i + 3] * MDS_MATRIX[j][i + 3];
                self.push_pi(Fq::zero());
                let z2 = self.cs.add(
                    values[i + 2],
                    values[i + 3],
                    MDS_MATRIX[j][i + 2],
                    MDS_MATRIX[j][i + 3],
                    -Fq::one(),
                    q_c,
                    Fq::zero(),
                );

                self.push_pi(Fq::zero());
                z3 = self.cs.add(
                    z1,
                    z2,
                    Fq::one(),
                    Fq::one(),
                    -Fq::one(),
                    Fq::zero(),
                    Fq::zero(),
                );
            }

            // TODO - Replace by compiler constant evaluation
            if WIDTH < 4 {
                for k in 0..WIDTH {
                    self.push_pi(Fq::zero());
                    product[k] = self.cs.add(
                        product[k],
                        values[j],
                        Fq::one(),
                        MDS_MATRIX[k][j],
                        -Fq::one(),
                        constants[j] * MDS_MATRIX[k][j],
                        Fq::zero(),
                    );
                }
            } else if WIDTH & 1 == 1 {
                self.push_pi(Fq::zero());
                product[j] = self.cs.add(
                    z3,
                    values[WIDTH - 1],
                    Fq::one(),
                    MDS_MATRIX[j][WIDTH - 1],
                    -Fq::one(),
                    constants[WIDTH - 1] * MDS_MATRIX[j][WIDTH - 1],
                    Fq::zero(),
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
        I: Iterator<Item = &'b Fq>,
    {
        #[cfg(feature = "trace")]
        let circuit_size = self.cs.circuit_size();

        words.iter_mut().for_each(|w| {
            let p = constants
                .next()
                .cloned()
                .expect("Hades252 out of ARK constants");

            self.push_pi(Fq::zero());
            *w = self.cs.add(
                *w,
                self.zero,
                Fq::one(),
                Fq::zero(),
                -Fq::one(),
                p,
                Fq::zero(),
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
            .for_each(|(i, e)| *e = self.cs.add_input(Fq::from((i + 1) as u8)));

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
    use crate::{Bls12_381, Fq, GadgetStrategy, ScalarStrategy, Strategy, WIDTH};

    use ff_fft::EvaluationDomain;
    use merlin::Transcript;
    use num_traits::{One, Zero};
    use plonk::cs::composer::StandardComposer;
    use plonk::cs::constraint_system::Variable;
    use plonk::cs::proof::Proof;
    use plonk::cs::{Composer, PreProcessedCircuit};
    use plonk::srs;
    use poly_commit::kzg10::{Powers, VerifierKey};
    use rand::Rng;

    const TEST_PI_SIZE: usize = super::PI_SIZE + WIDTH + 3;

    fn perm(values: &mut [Fq]) {
        let mut strategy = ScalarStrategy::new();
        strategy.perm(values);
    }

    fn gen_transcript() -> Transcript {
        Transcript::new(b"hades-plonk")
    }

    fn circuit(
        domain: &EvaluationDomain<Fq>,
        ck: &Powers<Bls12_381>,
        x: &[Fq],
        h: &[Fq],
    ) -> (
        Transcript,
        PreProcessedCircuit<Bls12_381>,
        StandardComposer<Bls12_381>,
        [Fq; TEST_PI_SIZE],
    ) {
        let mut transcript = gen_transcript();
        let mut composer: StandardComposer<Bls12_381> = StandardComposer::new();

        let mut pi = [Fq::zero(); TEST_PI_SIZE];
        let mut x_var: [Variable; WIDTH] = unsafe { [std::mem::zeroed(); WIDTH] };
        x.iter()
            .zip(x_var.iter_mut())
            .for_each(|(x, v)| *v = composer.add_input(*x));

        let (composer, pi_iter) = GadgetStrategy::hades_gadget(composer, pi.iter_mut(), &mut x_var);
        assert_eq!(super::PI_SIZE, composer.circuit_size());

        let (mut composer, mut pi_iter) =
            GadgetStrategy::constrain_gadget(composer, pi_iter, &x_var, h);

        (0..3).for_each(|_| {
            pi_iter
                .next()
                .map(|s| *s = Fq::zero())
                .expect("Public inputs iterator depleted");

            composer.add_dummy_constraints();
        });

        let circuit = composer.preprocess(&ck, &mut transcript, &domain);
        (transcript, circuit, composer, pi)
    }

    fn prove(
        domain: &EvaluationDomain<Fq>,
        ck: &Powers<Bls12_381>,
        x: &[Fq],
        h: &[Fq],
    ) -> (Proof<Bls12_381>, [Fq; TEST_PI_SIZE]) {
        let (mut transcript, circuit, mut composer, pi) = circuit(domain, ck, x, h);
        let proof = composer.prove(&ck, &circuit, &mut transcript);
        (proof, pi)
    }

    fn verify(
        transcript: &mut Transcript,
        circuit: &PreProcessedCircuit<Bls12_381>,
        vk: &VerifierKey<Bls12_381>,
        proof: &Proof<Bls12_381>,
        pi: &[Fq],
    ) -> bool {
        proof.verify(&circuit, transcript, vk, &pi.to_vec())
    }

    #[test]
    fn hades_preimage() {
        const CAPACITY: usize = 4096;

        let public_parameters = srs::setup(CAPACITY, &mut rand::thread_rng());
        let (ck, vk) = srs::trim(&public_parameters, CAPACITY).unwrap();
        let domain: EvaluationDomain<Fq> = EvaluationDomain::new(CAPACITY).unwrap();

        let e = [Fq::from(5000u64); WIDTH];
        let mut e_perm = [Fq::from(5000u64); WIDTH];
        perm(&mut e_perm);

        let (transcript, circuit, _, _) = circuit(&domain, &ck, &e, &e_perm);

        let x_scalar = Fq::from(31u64);
        let mut x = [Fq::zero(); WIDTH];
        x[1] = x_scalar;
        let mut h = [Fq::zero(); WIDTH];
        h.copy_from_slice(&x);
        perm(&mut h);

        let y_scalar = Fq::from(30u64);
        let mut y = [Fq::zero(); WIDTH];
        y[1] = y_scalar;
        let mut i = [Fq::zero(); WIDTH];
        i.copy_from_slice(&y);
        perm(&mut i);

        let (proof, pi) = prove(&domain, &ck, &x, &h);
        assert!(verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        let (proof, pi) = prove(&domain, &ck, &y, &i);
        assert!(verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        // Wrong pre-image
        let (proof, pi) = prove(&domain, &ck, &y, &h);
        assert!(!verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        // Wrong public image
        let (proof, pi) = prove(&domain, &ck, &x, &i);
        assert!(!verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        // Inconsistent public image
        let (proof, _) = prove(&domain, &ck, &x, &h);
        assert!(!verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));
    }

    #[test]
    fn poseidon_slice() {
        let public_parameters = srs::setup(4096 * 32, &mut rand::thread_rng());
        let (ck, vk) = srs::trim(&public_parameters, 4096 * 32).unwrap();
        let domain: EvaluationDomain<Fq> = EvaluationDomain::new(4096 * 32).unwrap();

        // Generate circuit
        let mut base_transcript = gen_transcript();
        let mut composer: StandardComposer<Bls12_381> = StandardComposer::new();

        const BITS: usize = WIDTH * 20 - 19;
        const SLICE_PI_SIZE: usize = super::PI_SIZE * (1 + BITS / 2);

        let mut pi = vec![Fq::zero(); SLICE_PI_SIZE];

        let data: Vec<Fq> = (0..BITS).map(|_| (&mut rand::thread_rng()).gen()).collect();
        let result = ScalarStrategy::new().poseidon_slice(data.as_slice());
        let result = composer.add_input(result);

        let vars: Vec<Variable> = data.iter().map(|d| composer.add_input(*d)).collect();
        let (mut composer, _, x) =
            GadgetStrategy::poseidon_slice_gadget(composer, pi.iter_mut(), &vars);

        let zero = composer.add_input(Fq::zero());
        composer.add_gate(
            result,
            x,
            zero,
            -Fq::one(),
            Fq::one(),
            Fq::one(),
            Fq::zero(),
            Fq::zero(),
        );

        composer.add_dummy_constraints();

        let preprocessed_circuit = composer.preprocess(&ck, &mut base_transcript, &domain);

        // Prove
        let mut transcript = gen_transcript();
        let mut composer: StandardComposer<Bls12_381> = StandardComposer::new();

        let mut pi = vec![Fq::zero(); SLICE_PI_SIZE];

        let data: Vec<Fq> = (0..BITS).map(|_| (&mut rand::thread_rng()).gen()).collect();
        let result = ScalarStrategy::new().poseidon_slice(data.as_slice());
        let result = composer.add_input(result);

        let vars: Vec<Variable> = data.iter().map(|d| composer.add_input(*d)).collect();
        let (mut composer, _, x) =
            GadgetStrategy::poseidon_slice_gadget(composer, pi.iter_mut(), &vars);

        let zero = composer.add_input(Fq::zero());
        composer.add_gate(
            result,
            x,
            zero,
            -Fq::one(),
            Fq::one(),
            Fq::one(),
            Fq::zero(),
            Fq::zero(),
        );

        composer.add_dummy_constraints();

        let circuit = composer.preprocess(&ck, &mut transcript, &domain);
        let proof = composer.prove(&ck, &circuit, &mut transcript);

        // Verify
        let mut transcript = base_transcript.clone();
        assert!(proof.verify(&preprocessed_circuit, &mut transcript, &vk, &pi));
    }
}
