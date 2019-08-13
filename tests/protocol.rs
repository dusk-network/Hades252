extern crate hades252;

use bulletproofs::r1cs::{Prover, R1CSProof, Verifier};
use bulletproofs::{BulletproofGens, PedersenGens};
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

use hades252::hash::Hash;
use hades252::permutation::Permutation;

#[test]
fn main() {
    // Common Bulletproof Parameters
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(1000, 1);

    // Common poseidon parameters
    let width = 4;
    let full_rounds = 2;
    let partial_rounds = 3;

    // Prover makes proof
    // Proof claims that the prover knows the pre-image to the digest produced from the poseidon hash function
    let (digest, proof, commitments) = make_proof(width, full_rounds, partial_rounds, &pc_gens, &bp_gens);

    // Verify verifies proof
    verify_proof(width, full_rounds, partial_rounds, &pc_gens, &bp_gens, digest, proof, commitments)
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

    // Build CS
    h.result_gadget(digest, vars, &mut prover).unwrap();

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

    h.result_gadget(digest, vars, &mut verifier);

    verifier.verify(&proof, &pc_gens, &bp_gens).unwrap()
}