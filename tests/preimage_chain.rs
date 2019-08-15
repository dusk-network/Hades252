#![feature(test)]

extern crate hades252;
use hades252::hash::Hash;
use hades252::permutation::Permutation;

use bulletproofs::r1cs::{ConstraintSystem, LinearCombination, Prover, R1CSProof, Verifier};
use bulletproofs::{BulletproofGens, PedersenGens};
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

/*

let x = H(y)
let z = H(x)
let d = H(z)

Tests whether given `x` , we have the correct `d` value
*/

#[test]
fn test_preimage_chain() {
    // Common Bulletproof Parameters
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(4096, 1);

    let input = Scalar::from(21 as u64);
    // Prover makes proof
    // Proof claims that the prover knows the pre-image to the digest produced from the poseidon hash function
    let (proof, commitments, x, d, z) = make_proof(&pc_gens, &bp_gens, input);

    // Verify verifies proof
    verify_proof(&pc_gens, &bp_gens, proof, commitments, x, d, z)
}

fn make_proof(
    pc_gens: &PedersenGens,
    bp_gens: &BulletproofGens,
    y: Scalar,
) -> (R1CSProof, Vec<CompressedRistretto>, Scalar, Scalar, Scalar) {
    let mut h = Hash::new();

    // x = H(y)
    h.input(y);
    let x = h.result().unwrap();
    h.reset();

    // z = H(x)
    h.input(x);
    let z = h.result().unwrap();
    h.reset();

    // d = H(z)
    h.input(z);
    let d = h.result().unwrap();
    h.reset();

    // Setup Prover
    let mut prover_transcript = Transcript::new(b"");
    let mut rng = rand::thread_rng();
    let mut prover = Prover::new(&pc_gens, &mut prover_transcript);

    // Commit High level variables
    let (com, vars): (Vec<_>, Vec<_>) = [y]
        .iter()
        .map(|input| prover.commit(*input, Scalar::random(&mut rng)))
        .unzip();

    // Convert variables into linear combinations
    let lcs: Vec<LinearCombination> = vars.iter().map(|&x| x.into()).collect();

    // Build CS
    preimage_chain_gadget(lcs[0].clone(), x.into(), z.into(), d.into(), &mut prover);

    // Prove
    let proof = prover.prove(&bp_gens).unwrap();

    (proof, com, x, z, d)
}

fn verify_proof(
    pc_gens: &PedersenGens,
    bp_gens: &BulletproofGens,
    proof: R1CSProof,
    commitments: Vec<CompressedRistretto>,
    x: Scalar,
    z: Scalar,
    d: Scalar,
) {
    // Verify results
    let mut verifier_transcript = Transcript::new(b"");
    let mut verifier = Verifier::new(&mut verifier_transcript);

    let vars: Vec<_> = commitments.iter().map(|V| verifier.commit(*V)).collect();;

    let mut h = Hash::new();

    // Convert variables into linear combinations
    let lcs: Vec<LinearCombination> = vars.iter().map(|&x| x.into()).collect();

    // Add preimage gadget
    preimage_chain_gadget(lcs[0].clone(), x.into(), z.into(), d.into(), &mut verifier);

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

fn preimage_chain_gadget(
    pre_image_y: LinearCombination,
    x_lc: LinearCombination,
    z_lc: LinearCombination,
    d_lc: LinearCombination,
    cs: &mut dyn ConstraintSystem,
) {
    let mut h = Hash::new();

    // x = H(y)
    h.input_lc(pre_image_y).unwrap();
    let x = h.result_gadget(cs).unwrap();
    cs.constrain(x_lc - x.clone());

    h.reset();

    // z = H(x)
    h.input_lc(x).unwrap();
    let z = h.result_gadget(cs).unwrap();
    cs.constrain(z_lc - z.clone());
    
    h.reset();

    // d = H(z)
    h.input_lc(z).unwrap();
    let d = h.result_gadget(cs).unwrap();
    cs.constrain(d - d_lc);
}
