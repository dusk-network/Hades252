use crate::{mds_matrix::MDS_MATRIX, Scalar, WIDTH};

use super::Strategy;

use plonk::constraint_system::{
    linear_combination::Variable, standard::composer::StandardComposer,
};

/// Implements a Hades252 strategy for `Variable` as input values.
/// Requires a reference to a `ConstraintSystem`.
pub struct GadgetStrategy<'a> {
    /// A reference to the constraint system used by the gadgets
    pub cs: &'a mut StandardComposer,
    /// If this flag is set, then the struct will not interact with the composer, and will only
    /// create the public inputs
    pub dummy_mode: bool,
    /// Public inputs created during the processing
    pub public_inputs: Vec<Scalar>,
}

impl<'a> GadgetStrategy<'a> {
    /// Constructs a new `GadgetStrategy` with the constraint system.
    pub fn new(cs: &'a mut StandardComposer, dummy_mode: bool) -> Self {
        GadgetStrategy {
            cs,
            dummy_mode,
            public_inputs: vec![],
        }
    }
}

impl<'a> Strategy<Variable> for GadgetStrategy<'a> {
    fn quintic_s_box(&mut self, value: &mut Variable) {
        self.public_inputs.extend_from_slice(&[Scalar::zero(); 4]);
        if self.dummy_mode {
            return;
        }

        let v = *value;

        (0..4).for_each(|_| {
            *value = self.cs.mul(Scalar::one(), *value, v, Scalar::zero());
        });
    }

    fn mul_matrix(&mut self, values: &mut [Variable]) {
        self.public_inputs
            .extend_from_slice(&[Scalar::zero(); WIDTH * WIDTH * 2]);
        if self.dummy_mode {
            return;
        }

        let zero = self.cs.add_input(Scalar::zero());
        let mut product = [zero; WIDTH];

        for j in 0..WIDTH {
            for k in 0..WIDTH {
                let a = self.cs.add_input(MDS_MATRIX[j][k]);
                let b = values[k];
                let o = self.cs.mul(Scalar::one(), a, b, Scalar::zero());

                let a = product[j];
                let b = o;
                let o = self
                    .cs
                    .add((Scalar::one(), a), (Scalar::one(), b), Scalar::zero());

                product[j] = o;
            }
        }

        values.copy_from_slice(&product);
    }

    fn add_round_key<'b, I>(&mut self, constants: &mut I, words: &mut [Variable])
    where
        I: Iterator<Item = &'b Scalar>,
    {
        words.iter_mut().for_each(|w| {
            let p = constants.next().cloned().unwrap_or_default();

            self.public_inputs.push(p);
            if !self.dummy_mode {
                *w = self.cs.add((Scalar::one(), *w), (Scalar::zero(), *w), p);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{GadgetStrategy, Scalar, ScalarStrategy, Strategy, WIDTH};

    use merlin::Transcript;
    use plonk::{
        commitment_scheme::kzg10::{ProverKey, VerifierKey, SRS},
        constraint_system::{
            linear_combination::Variable,
            standard::{composer::StandardComposer, proof::Proof, Composer, PreProcessedCircuit},
        },
        fft::EvaluationDomain,
    };

    fn perm(values: &mut [Scalar]) {
        let mut strategy = ScalarStrategy::new();
        strategy.perm(values);
    }

    fn gen_transcript() -> Transcript {
        Transcript::new(b"hades-plonk")
    }

    fn hades_gadget(
        composer: &mut StandardComposer,
        x: Option<&[Scalar]>,
        h: &[Scalar],
        dummy_mode: bool,
    ) -> Vec<Scalar> {
        let zero = composer.add_input(Scalar::zero());
        let mut x_var: Vec<Variable> = x
            .unwrap_or(&[Scalar::one(); WIDTH])
            .iter()
            .map(|s| composer.add_input(*s))
            .collect();

        let mut strategy = GadgetStrategy::new(composer, dummy_mode);
        strategy.perm(x_var.as_mut_slice());

        let mut public_inputs = strategy.public_inputs;

        public_inputs.extend_from_slice(h);
        public_inputs.extend_from_slice(&[Scalar::zero(); 6]);
        if !dummy_mode {
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

        public_inputs
    }

    fn circuit(
        domain: &EvaluationDomain,
        ck: &ProverKey,
        h: &[Scalar],
    ) -> (Transcript, PreProcessedCircuit) {
        let mut transcript = gen_transcript();
        let mut composer = StandardComposer::new();

        hades_gadget(&mut composer, None, h, false);

        let circuit = composer.preprocess(&ck, &mut transcript, &domain);

        (transcript, circuit)
    }

    fn prove(domain: &EvaluationDomain, ck: &ProverKey, x: &[Scalar], h: &[Scalar]) -> Proof {
        let mut transcript = gen_transcript();
        let mut composer = StandardComposer::new();

        hades_gadget(&mut composer, Some(x), h, false);

        let preprocessed_circuit = composer.preprocess(&ck, &mut transcript, &domain);
        composer.prove(&ck, &preprocessed_circuit, &mut transcript)
    }

    fn verify(
        transcript: &mut Transcript,
        circuit: &PreProcessedCircuit,
        vk: &VerifierKey,
        proof: &Proof,
        pi: &[Scalar],
    ) -> bool {
        proof.verify(&circuit, transcript, vk, &pi.to_vec())
    }

    #[test]
    fn hades_preimage() {
        let public_parameters = SRS::setup(8192, &mut rand::thread_rng()).unwrap();
        let (ck, vk) = public_parameters.trim(8192).unwrap();
        let domain = EvaluationDomain::new(2100).unwrap();

        let mut e = [Scalar::from(5000u64); WIDTH];
        perm(&mut e);

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

        let proof = prove(&domain, &ck, &x, &h);
        let pi = hades_gadget(&mut StandardComposer::new(), None, &h, true);
        assert!(verify(
            &mut transcript.clone(),
            &circuit,
            &vk,
            &proof,
            pi.as_slice()
        ));

        let proof = prove(&domain, &ck, &y, &i);
        let pi = hades_gadget(&mut StandardComposer::new(), None, &i, true);
        assert!(verify(
            &mut transcript.clone(),
            &circuit,
            &vk,
            &proof,
            pi.as_slice()
        ));

        // Wrong pre-image
        let proof = prove(&domain, &ck, &y, &h);
        let pi = hades_gadget(&mut StandardComposer::new(), None, &h, true);
        assert!(!verify(
            &mut transcript.clone(),
            &circuit,
            &vk,
            &proof,
            pi.as_slice()
        ));

        // Wrong public image
        let proof = prove(&domain, &ck, &x, &i);
        let pi = hades_gadget(&mut StandardComposer::new(), None, &i, true);
        assert!(!verify(
            &mut transcript.clone(),
            &circuit,
            &vk,
            &proof,
            pi.as_slice()
        ));

        // Inconsistent public image
        let proof = prove(&domain, &ck, &x, &h);
        let pi = hades_gadget(&mut StandardComposer::new(), None, &i, true);
        assert!(!verify(
            &mut transcript.clone(),
            &circuit,
            &vk,
            &proof,
            pi.as_slice()
        ));
    }
}
