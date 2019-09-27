#![feature(test)]

extern crate hades252;
use hades252::errors::PermError;
use hades252::scalar;

use bulletproofs::r1cs::{
    ConstraintSystem, LinearCombination, Prover, R1CSError, R1CSProof, Verifier,
};
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

    let input = Scalar::from(21_u64);
    // Prover makes proof
    // Proof claims that the prover knows the pre-image to the digest produced from the poseidon hash function
    let (proof, commitments, x, d, z) = make_proof(&pc_gens, &bp_gens, input).unwrap();

    // Verify verifies proof
    assert!(verify_proof(&pc_gens, &bp_gens, proof, commitments, x, d, z).is_ok())
}

fn make_proof(
    pc_gens: &PedersenGens,
    bp_gens: &BulletproofGens,
    y: Scalar,
) -> Result<(R1CSProof, Vec<CompressedRistretto>, Scalar, Scalar, Scalar), PermError> {
    // x = H(y)
    let x = scalar::hash(&[y])?;

    // z = H(x)
    let z = scalar::hash(&[x])?;

    // d = H(z)
    let d = scalar::hash(&[z])?;

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
    preimage_chain_gadget(lcs[0].clone(), x.into(), z.into(), d.into(), &mut prover)?;

    // Prove
    let proof = prover.prove(&bp_gens).unwrap();

    Ok((proof, com, x, z, d))
}

fn verify_proof(
    pc_gens: &PedersenGens,
    bp_gens: &BulletproofGens,
    proof: R1CSProof,
    commitments: Vec<CompressedRistretto>,
    x: Scalar,
    z: Scalar,
    d: Scalar,
) -> Result<(), R1CSError> {
    // Verify results
    let mut verifier_transcript = Transcript::new(b"");
    let mut verifier = Verifier::new(&mut verifier_transcript);
    let mut rng = rand::thread_rng();

    let vars: Vec<_> = commitments
        .iter()
        .map(|v_point| verifier.commit(*v_point))
        .collect();

    // Convert variables into linear combinations
    let lcs: Vec<LinearCombination> = vars.iter().map(|&x| x.into()).collect();

    // Add preimage gadget
    preimage_chain_gadget(lcs[0].clone(), x.into(), z.into(), d.into(), &mut verifier).unwrap();

    verifier.verify(&proof, &pc_gens, &bp_gens, &mut rng)
}

fn preimage_chain_gadget(
    pre_image_y: LinearCombination,
    x_lc: LinearCombination,
    z_lc: LinearCombination,
    d_lc: LinearCombination,
    cs: &mut dyn ConstraintSystem,
) -> Result<(), PermError> {
    use hades252::linear_combination::hash;

    // x = H(y)
    let x = hash(cs, &[pre_image_y]).unwrap();
    cs.constrain(x_lc - x.clone());

    // z = H(x)
    let z = hash(cs, &[x]).unwrap();
    cs.constrain(z_lc - z.clone());

    // d = H(z)
    let d = hash(cs, &[z]).unwrap();
    cs.constrain(d - d_lc);

    Ok(())
}
