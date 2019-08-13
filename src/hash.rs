use crate::errors::PermError;
use crate::permutation::Permutation;
use bulletproofs::r1cs::{ConstraintSystem, LinearCombination, Variable};
use curve25519_dalek::scalar::Scalar;

// hash is a thin layer over the permutation struct
pub struct Hash {
    perm: Permutation,
}

impl Hash {
    pub fn new() -> Self {
        Hash::with_perm(Permutation::default())
    }

    pub fn with_perm(perm: Permutation) -> Self {
        let mut h = Hash { perm: perm };
        h.reset();
        h
    }

    pub fn input(&mut self, s: Scalar) -> Result<(), PermError> {
        self.perm.input(s)
    }

    pub fn data(&self) -> Vec<Scalar> {
        self.perm.data.clone()
    }

    pub fn reset(&mut self) {
        self.perm.reset();
        self.perm.data.push(Scalar::from(0 as u8))
    }

    pub fn input_bytes(&mut self, bytes: &[u8]) -> Result<(), PermError> {
        self.perm.input_bytes(bytes)
    }
    pub fn inputs(&mut self, scalars: Vec<Scalar>) -> Result<(), PermError> {
        self.perm.inputs(scalars)
    }

    fn pad(&mut self) {
        let pad_amount = self.perm.width_left();
        let zero = Scalar::from(0 as u8);
        let zeroes = vec![zero; pad_amount];

        self.perm.data.extend(zeroes)
    }

    pub fn result(&mut self) -> Option<Scalar> {
        // Pad remaining width with zero
        self.pad();

        // Apply permutation
        let words = self.perm.result().ok();
        match words {
            Some(words) => Some(words[1]),
            None => None,
        }
    }

    pub fn result_gadget(
        &mut self,
        input: Vec<LinearCombination>,
        cs: &mut dyn ConstraintSystem,
    ) -> Option<LinearCombination> {
        // Pad remaining width with zero
        self.pad();

        // Apply permutation
        let words = self.perm.constrain_result(cs, input).ok();
        match words {
            Some(words) => Some(words[1].clone()),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hex::encode;

    extern crate test;
    use test::Bencher;

    use bulletproofs::r1cs::{Prover, R1CSProof, Verifier};
    use bulletproofs::{BulletproofGens, PedersenGens};
    use curve25519_dalek::ristretto::CompressedRistretto;
    use curve25519_dalek::scalar::Scalar;
    use merlin::Transcript;

    #[test]
    fn test_hash_reset() {
        let mut h = Hash::new();
        h.input_bytes(b"hello").unwrap();
        h.input_bytes(b"world").unwrap();
        let digest = h.result().unwrap();
        assert_eq!(
            "d92a019379b8a2dff3b37d4b3b59e688388912c06ffd31693e0dadcbf3595506",
            hex::encode(digest.to_bytes())
        );

        h.reset();

        h.input_bytes(b"hello").unwrap();
        h.input_bytes(b"world").unwrap();
        let digest = h.result().unwrap();
        assert_eq!(
            "d92a019379b8a2dff3b37d4b3b59e688388912c06ffd31693e0dadcbf3595506",
            hex::encode(digest.to_bytes())
        );
    }

    #[bench]
    fn bench_hash3_1(b: &mut Bencher) {
        let mut h = Hash::new();
        h.input_bytes(b"a").unwrap();
        h.input_bytes(b"b").unwrap();
        h.input_bytes(b"c").unwrap();

        b.iter(|| h.result().unwrap());
    }

    #[bench]
    fn bench_prove(b: &mut Bencher) {
        // Common Bulletproof Parameters
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(1000, 1);

        // Common poseidon parameters
        let width = 9;
        let full_rounds = 8;
        let partial_rounds = 10;

        b.iter(|| make_proof(width, full_rounds, partial_rounds, &pc_gens, &bp_gens))
    }
    #[bench]
    fn bench_verify(b: &mut Bencher) {
        // Common Bulletproof Parameters
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(1000, 1);

        // Common poseidon parameters
        let width = 4;
        let full_rounds = 8;
        let partial_rounds = 59;

        // Prover makes proof
        // Proof claims that the prover knows the pre-image to the digest produced from the poseidon hash function
        let (digest, proof, commitments) =
            make_proof(width, full_rounds, partial_rounds, &pc_gens, &bp_gens);

        b.iter(|| {
            verify_proof(
                width,
                full_rounds,
                partial_rounds,
                &pc_gens,
                &bp_gens,
                digest,
                proof.clone(),
                commitments.clone(),
            )
        })
    }

    fn make_proof(
        width: usize,
        full_round: usize,
        partial_round: usize,
        pc_gens: &PedersenGens,
        bp_gens: &BulletproofGens,
    ) -> (Scalar, R1CSProof, Vec<CompressedRistretto>) {
        // Setup hash object; adding in our input
        let perm = Permutation::new(width, full_round, partial_round).unwrap();
        let mut h = Hash::with_perm(perm);
        h.input_bytes(b"hello").unwrap();
        h.input_bytes(b"world").unwrap();
        let digest = h.result().unwrap();

        // Setup Prover
        let mut prover_transcript = Transcript::new(b"");
        let mut rng = rand::thread_rng();
        let mut prover = Prover::new(&pc_gens, &mut prover_transcript);

        // Commit High level variables
        let (com, vars): (Vec<_>, Vec<_>) = h
            .data()
            .iter()
            .map(|input| prover.commit(*input, Scalar::random(&mut rng)))
            .unzip();

        // Convert variables into linear combinations
        let lcs : Vec<LinearCombination> = vars.iter().map(|&x| x.into()).collect();

        // Build CS
        let results = h.result_gadget(lcs, &mut prover).unwrap();

        // preimage gadget
        preimage_gadget(digest, results, &mut prover);

        // Prove
        let proof = prover.prove(&bp_gens).unwrap();

        (digest, proof, com)
    }

    fn verify_proof(
        width: usize,
        full_round: usize,
        partial_round: usize,
        pc_gens: &PedersenGens,
        bp_gens: &BulletproofGens,
        digest: Scalar,
        proof: R1CSProof,
        commitments: Vec<CompressedRistretto>,
    ) {
        // Verify results
        let mut verifier_transcript = Transcript::new(b"");
        let mut verifier = Verifier::new(&mut verifier_transcript);

        let vars: Vec<_> = commitments.iter().map(|V| verifier.commit(*V)).collect();;

        let perm = Permutation::new(width, full_round, partial_round).unwrap();

        let mut h = Hash::with_perm(perm);

        // Convert variables into linear combinations
        let lcs : Vec<LinearCombination> = vars.iter().map(|&x| x.into()).collect();

        let result = h.result_gadget(lcs, &mut verifier).unwrap();

        // constrain preimage to be digest
        preimage_gadget(digest, result, &mut verifier);

        verifier.verify(&proof, &pc_gens, &bp_gens).unwrap()
    }

    fn preimage_gadget(
        digest: Scalar,
        gadget_digest: LinearCombination,
        cs: &mut dyn ConstraintSystem,
    ) {
        let digest_lc: LinearCombination = digest.into();
        cs.constrain(digest_lc - gadget_digest)
    }
}
