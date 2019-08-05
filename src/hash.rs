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
        digest: Scalar,
        input: Vec<Variable>,
        cs: &mut dyn ConstraintSystem,
    ) -> Option<LinearCombination> {
        // Pad remaining width with zero
        self.pad();

        // Apply permutation
        let words = self.perm.constrain_result(cs, input).ok();
        match words {
            Some(words) => {
                // constrain output to be digest
                cs.constrain(words[1].clone() - digest);
                Some(words[1].clone())
            }
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
    fn bench_hash2_1(b: &mut Bencher) {
        let mut h = Hash::new();
        h.input_bytes(b"hello").unwrap();
        h.input_bytes(b"world").unwrap();

        b.iter(|| h.result().unwrap());
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
    fn bench_hash4_1(b: &mut Bencher) {
        let mut h = Hash::new();
        h.input_bytes(b"a").unwrap();
        h.input_bytes(b"b").unwrap();
        h.input_bytes(b"c").unwrap();
        h.input_bytes(b"d").unwrap();

        b.iter(|| h.result().unwrap());
    }
    #[bench]
    fn bench_hash5_1(b: &mut Bencher) {
        let mut h = Hash::new();
        h.input_bytes(b"a").unwrap();
        h.input_bytes(b"b").unwrap();
        h.input_bytes(b"c").unwrap();
        h.input_bytes(b"d").unwrap();
        h.input_bytes(b"e").unwrap();

        b.iter(|| h.result().unwrap());
    }
}

// fn range_proof_helper(v_val: u64, n: usize) -> Result<(), R1CSError> {
//     // Common
//     let pc_gens = PedersenGens::default();
//     let bp_gens = BulletproofGens::new(128, 1);

//     // Prover's scope
//     let (proof, commitment) = {
//         // Prover makes a `ConstraintSystem` instance representing a range proof gadget
//         let mut prover_transcript = Transcript::new(b"RangeProofTest");
//         let mut rng = rand::thread_rng();

//         let mut prover = Prover::new(&pc_gens, &mut prover_transcript);

//         let (com, var) = prover.commit(v_val.into(), Scalar::random(&mut rng));
//         assert!(range_proof(&mut prover, var.into(), Some(v_val), n).is_ok());

//         let proof = prover.prove(&bp_gens)?;

//         (proof, com)
//     };

//     // Verifier makes a `ConstraintSystem` instance representing a merge gadget
//     let mut verifier_transcript = Transcript::new(b"RangeProofTest");
//     let mut verifier = Verifier::new(&mut verifier_transcript);

//     let var = verifier.commit(commitment);

//     // Verifier adds constraints to the constraint system
//     assert!(range_proof(&mut verifier, var.into(), None, n).is_ok());

//     // Verifier verifies proof
//     Ok(verifier.verify(&proof, &pc_gens, &bp_gens)?)
// }
