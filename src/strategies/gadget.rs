use super::Strategy;
use crate::{mds_matrix::MDS_MATRIX, Curve, Scalar, PARTIAL_ROUNDS, TOTAL_FULL_ROUNDS, WIDTH};

use num_traits::{One, Zero};
use plonk::cs::{composer::StandardComposer, constraint_system::Variable};

/// Size of the generated public inputs for the permutation gadget
pub const PI_SIZE: usize =
    WIDTH * (TOTAL_FULL_ROUNDS + PARTIAL_ROUNDS) + 65 * TOTAL_FULL_ROUNDS + 53 * PARTIAL_ROUNDS;

/// Implements a Hades252 strategy for `Variable` as input values.
/// Requires a reference to a `ConstraintSystem`.
pub struct GadgetStrategy<'a, P>
where
    P: Iterator<Item = &'a mut Scalar>,
{
    /// A reference to the constraint system used by the gadgets
    pub cs: StandardComposer<Curve>,
    /// Mutable iterator over the public inputs
    pub pi_iter: P,
}

impl<'a, P> GadgetStrategy<'a, P>
where
    P: Iterator<Item = &'a mut Scalar>,
{
    /// Constructs a new `GadgetStrategy` with the constraint system.
    pub fn new(cs: StandardComposer<Curve>, pi_iter: P) -> Self {
        GadgetStrategy { cs, pi_iter }
    }

    /// Return the inner iterator over public inputs
    pub fn into_inner(self) -> (StandardComposer<Curve>, P) {
        (self.cs, self.pi_iter)
    }

    /// Perform the pre-image zk proof
    pub fn hades_gadget(
        mut composer: StandardComposer<Curve>,
        pi: P,
        x: Option<&[Scalar]>,
        h: &[Scalar],
    ) -> (StandardComposer<Curve>, P) {
        let zero = composer.add_input(Scalar::zero());
        let mut x_var: Vec<Variable> = x
            .unwrap_or(&[Scalar::one(); WIDTH])
            .iter()
            .map(|s| composer.add_input(*s))
            .collect();

        let mut strategy = GadgetStrategy::new(composer, pi);
        strategy.perm(x_var.as_mut_slice());

        let (mut composer, mut pi) = strategy.into_inner();

        x_var.iter().zip(h.iter()).for_each(|(a, b)| {
            pi.next()
                .map(|p| *p = *b)
                .expect("Not enough public inputs");

            composer.add_gate(
                *a,
                zero,
                zero,
                -Scalar::one(),
                Scalar::one(),
                Scalar::one(),
                Scalar::zero(),
                *b,
            );
        });

        (0..3).for_each(|_| {
            pi.next()
                .map(|p| *p = Scalar::zero())
                .expect("Not enough public inputs");
            composer.add_dummy_constraints();
        });

        (composer, pi)
    }

    fn push_pi(&mut self, p: Scalar) {
        self.pi_iter
            .next()
            .map(|s| *s = p)
            .expect("Public inputs iterator depleted");
    }
}

impl<'a, P> Strategy<Variable> for GadgetStrategy<'a, P>
where
    P: Iterator<Item = &'a mut Scalar>,
{
    fn quintic_s_box(&mut self, value: &mut Variable) {
        let v = *value;

        (0..2).for_each(|_| {
            self.push_pi(Scalar::zero());
            *value = self.cs.mul(
                *value,
                *value,
                Scalar::one(),
                -Scalar::one(),
                Scalar::zero(),
                Scalar::zero(),
            )
        });

        self.push_pi(Scalar::zero());
        *value = self.cs.mul(
            *value,
            v,
            Scalar::one(),
            -Scalar::one(),
            Scalar::zero(),
            Scalar::zero(),
        );
    }

    fn mul_matrix(&mut self, values: &mut [Variable]) {
        let zero = self.cs.add_input(Scalar::zero());
        let mut product = [zero; WIDTH];

        for j in 0..WIDTH {
            for k in 0..WIDTH {
                self.push_pi(Scalar::zero());
                let a = self.cs.add_input(MDS_MATRIX[j][k]);
                let o = self.cs.mul(
                    a,
                    values[k],
                    Scalar::one(),
                    -Scalar::one(),
                    Scalar::zero(),
                    Scalar::zero(),
                );

                self.push_pi(Scalar::zero());
                product[j] = self.cs.add(
                    product[j],
                    o,
                    Scalar::one(),
                    Scalar::one(),
                    -Scalar::one(),
                    Scalar::zero(),
                    Scalar::zero(),
                );
            }
        }

        values.copy_from_slice(&product);
    }

    fn add_round_key<'b, I>(&mut self, constants: &mut I, words: &mut [Variable])
    where
        I: Iterator<Item = &'b Scalar>,
    {
        let zero = self.cs.add_input(Scalar::zero());

        words.iter_mut().for_each(|w| {
            let p = constants
                .next()
                .cloned()
                .expect("Hades252 out of ARK constants");

            self.push_pi(p);
            *w = self.cs.add(
                *w,
                zero,
                Scalar::one(),
                Scalar::zero(),
                -Scalar::one(),
                Scalar::zero(),
                p,
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{Curve, GadgetStrategy, Scalar, ScalarStrategy, Strategy, WIDTH};

    use ff_fft::EvaluationDomain;
    use merlin::Transcript;
    use num_traits::Zero;
    use plonk::{
        cs::{composer::StandardComposer, proof::Proof, Composer, PreProcessedCircuit},
        srs,
    };
    use poly_commit::kzg10::{Powers, VerifierKey};

    const TEST_PI_SIZE: usize = super::PI_SIZE + WIDTH + 3;

    fn perm(values: &mut [Scalar]) {
        let mut strategy = ScalarStrategy::new();
        strategy.perm(values);
    }

    fn gen_transcript() -> Transcript {
        Transcript::new(b"hades-plonk")
    }

    fn circuit(
        domain: &EvaluationDomain<Scalar>,
        ck: &Powers<Curve>,
        h: &[Scalar],
    ) -> (Transcript, PreProcessedCircuit<Curve>) {
        let mut transcript = gen_transcript();
        let composer: StandardComposer<Curve> = StandardComposer::new();

        let mut pi = [Scalar::zero(); TEST_PI_SIZE];
        let (mut composer, _) = GadgetStrategy::hades_gadget(composer, pi.iter_mut(), None, h);

        let circuit = composer.preprocess(&ck, &mut transcript, &domain);

        (transcript, circuit)
    }

    fn prove(
        domain: &EvaluationDomain<Scalar>,
        ck: &Powers<Curve>,
        pi: &mut [Scalar],
        x: &[Scalar],
        h: &[Scalar],
    ) -> Proof<Curve> {
        let mut transcript = gen_transcript();
        let composer: StandardComposer<Curve> = StandardComposer::new();

        let (mut composer, _) = GadgetStrategy::hades_gadget(composer, pi.iter_mut(), Some(x), h);

        let preprocessed_circuit = composer.preprocess(&ck, &mut transcript, &domain);
        composer.prove(&ck, &preprocessed_circuit, &mut transcript)
    }

    fn verify(
        transcript: &mut Transcript,
        circuit: &PreProcessedCircuit<Curve>,
        vk: &VerifierKey<Curve>,
        proof: &Proof<Curve>,
        pi: &[Scalar],
    ) -> bool {
        proof.verify(&circuit, transcript, vk, &pi.to_vec())
    }

    #[test]
    fn hades_preimage() {
        let public_parameters = srs::setup(8192, &mut rand::thread_rng());
        let (ck, vk) = srs::trim(&public_parameters, 8192).unwrap();
        let domain: EvaluationDomain<Scalar> = EvaluationDomain::new(4100).unwrap();

        let mut e = [Scalar::from(5000u64); WIDTH];
        perm(&mut e);

        let mut pi = [Scalar::zero(); TEST_PI_SIZE];
        let (transcript, circuit) = circuit(&domain, &ck, &e);

        let x_scalar = Scalar::from(31u64);
        let mut x = [Scalar::zero(); WIDTH];
        x[1] = x_scalar;
        let mut h = [Scalar::zero(); WIDTH];
        h.copy_from_slice(&x);
        perm(&mut h);

        let y_scalar = Scalar::from(30u64);
        let mut y = [Scalar::zero(); WIDTH];
        y[1] = y_scalar;
        let mut i = [Scalar::zero(); WIDTH];
        i.copy_from_slice(&y);
        perm(&mut i);

        let proof = prove(&domain, &ck, &mut pi, &x, &h);
        assert!(verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        let proof = prove(&domain, &ck, &mut pi, &y, &i);
        assert!(verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        // Wrong pre-image
        let proof = prove(&domain, &ck, &mut pi, &y, &h);
        assert!(!verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        // Wrong public image
        let proof = prove(&domain, &ck, &mut pi, &x, &i);
        assert!(!verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        // Inconsistent public image
        let pi_i = pi;
        let proof = prove(&domain, &ck, &mut pi, &x, &h);
        assert!(!verify(
            &mut transcript.clone(),
            &circuit,
            &vk,
            &proof,
            &pi_i
        ));
    }
}
