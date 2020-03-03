use crate::{mds_matrix::MDS_MATRIX, Curve, Scalar, WIDTH};

use super::Strategy;

use num_traits::{One, Zero};
use plonk::cs::{composer::StandardComposer, constraint_system::Variable};

/// Implements a Hades252 strategy for `Variable` as input values.
/// Requires a reference to a `ConstraintSystem`.
pub struct GadgetStrategy<'a> {
    /// A reference to the constraint system used by the gadgets
    pub cs: &'a mut StandardComposer<Curve>,
}

impl<'a> GadgetStrategy<'a> {
    /// Constructs a new `GadgetStrategy` with the constraint system.
    pub fn new(cs: &'a mut StandardComposer<Curve>) -> Self {
        GadgetStrategy { cs }
    }
}

impl<'a> Strategy<Variable> for GadgetStrategy<'a> {
    fn quintic_s_box(&mut self, value: &mut Variable) {
        let v_var = *value;
        let v = self.cs.eval(value);

        (0..4).for_each(|_| {
            let o = self.cs.eval(value) * v;
            let o_var = self.cs.add_input(o);

            self.cs.mul_gate(
                *value,
                v_var,
                o_var,
                Scalar::one(),
                -Scalar::one(),
                Scalar::zero(),
                Scalar::zero(),
            );

            *value = o_var;
        });
    }

    fn mul_matrix(&mut self, values: &mut [Variable]) {
        let zero = self.cs.add_input(Scalar::zero());
        let mut product = [zero; WIDTH];

        for j in 0..WIDTH {
            for k in 0..WIDTH {
                let a = self.cs.add_input(MDS_MATRIX[j][k]);
                let b = values[k];
                let o = MDS_MATRIX[j][k] * self.cs.eval(&b);
                let o = self.cs.add_input(o);

                self.cs.mul_gate(
                    a,
                    b,
                    o,
                    Scalar::one(),
                    -Scalar::one(),
                    Scalar::zero(),
                    Scalar::zero(),
                );

                let a = product[j];
                let b = o;
                let o = self.cs.eval(&a) + self.cs.eval(&b);
                let o = self.cs.add_input(o);
                self.cs.add_gate(
                    a,
                    b,
                    o,
                    Scalar::one(),
                    Scalar::one(),
                    -Scalar::one(),
                    Scalar::zero(),
                    Scalar::zero(),
                );

                product[j] = o;
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
            let p = constants.next().cloned().unwrap_or_default();

            let o = self.cs.eval(w) + p;
            let o = self.cs.add_input(o);

            self.cs.add_gate(
                *w,
                zero,
                o,
                Scalar::one(),
                Scalar::one(),
                -Scalar::one(),
                Scalar::zero(),
                p,
            );

            *w = o;
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{Curve, GadgetStrategy, Scalar, ScalarStrategy, Strategy, WIDTH};

    use ff_fft::EvaluationDomain;
    use merlin::Transcript;
    use num_traits::{One, Zero};
    use plonk::{
        cs::{
            composer::StandardComposer, constraint_system::Variable, proof::Proof, Composer,
            PreProcessedCircuit,
        },
        srs,
    };
    use poly_commit::kzg10::{Powers, VerifierKey};

    fn perm(values: &mut [Scalar]) {
        let mut strategy = ScalarStrategy::new();
        strategy.perm(values);
    }

    fn gen_transcript() -> Transcript {
        Transcript::new(b"hades-plonk")
    }

    fn hades_gadget(composer: &mut StandardComposer<Curve>, x: Option<&[Scalar]>, h: &[Scalar]) {
        let zero = composer.add_input(Scalar::zero());
        let mut x_var: Vec<Variable> = x
            .unwrap_or(&[Scalar::one(); WIDTH])
            .iter()
            .map(|s| composer.add_input(*s))
            .collect();

        let mut strategy = GadgetStrategy::new(composer);
        strategy.perm(x_var.as_mut_slice());

        x_var.iter().zip(h.iter()).for_each(|(a, b)| {
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

        composer.add_dummy_constraints();
        composer.add_dummy_constraints();
        composer.add_dummy_constraints();
    }

    fn circuit(
        domain: &EvaluationDomain<Scalar>,
        ck: &Powers<Curve>,
        h: &[Scalar],
    ) -> (Transcript, PreProcessedCircuit<Curve>, Vec<Scalar>) {
        let mut transcript = gen_transcript();
        let mut composer: StandardComposer<Curve> = StandardComposer::new();

        hades_gadget(&mut composer, None, h);

        let pi = composer.public_inputs().to_vec();
        let circuit = composer.preprocess(&ck, &mut transcript, &domain);

        (transcript, circuit, pi)
    }

    fn prove(
        domain: &EvaluationDomain<Scalar>,
        ck: &Powers<Curve>,
        x: &[Scalar],
        h: &[Scalar],
    ) -> Proof<Curve> {
        let mut transcript = gen_transcript();
        let mut composer: StandardComposer<Curve> = StandardComposer::new();

        hades_gadget(&mut composer, Some(x), h);

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

        let (transcript, circuit, mut pi) = circuit(&domain, &ck, &e);
        let pi_h_from = pi.len() - 6 - WIDTH;
        let pi_h_to = pi.len() - 6;

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

        let proof = prove(&domain, &ck, &x, &h);
        pi[pi_h_from..pi_h_to].copy_from_slice(&h);
        assert!(verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        let proof = prove(&domain, &ck, &y, &i);
        pi[pi_h_from..pi_h_to].copy_from_slice(&i);
        assert!(verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        // Wrong pre-image
        let proof = prove(&domain, &ck, &y, &h);
        pi[pi_h_from..pi_h_to].copy_from_slice(&h);
        assert!(!verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        // Wrong public image
        let proof = prove(&domain, &ck, &x, &i);
        pi[pi_h_from..pi_h_to].copy_from_slice(&i);
        assert!(!verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));

        // Inconsistent public image
        let proof = prove(&domain, &ck, &x, &h);
        pi[pi_h_from..pi_h_to].copy_from_slice(&i);
        assert!(!verify(&mut transcript.clone(), &circuit, &vk, &proof, &pi));
    }
}
