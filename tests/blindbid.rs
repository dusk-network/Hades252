#![feature(test)]

extern crate hades252;
use hades252::hash::Hash;

use bulletproofs::r1cs::{Prover, Verifier, ConstraintSystem};
use bulletproofs::{BulletproofGens, PedersenGens};
use curve25519_dalek::ristretto::CompressedRistretto;
use merlin::Transcript;
use rand::thread_rng;

use bulletproofs::r1cs::{LinearCombination, R1CSError, R1CSProof, Variable};
use curve25519_dalek::scalar::Scalar;

type ProofResult<T> = Result<T, R1CSError>;


pub fn prove(
    d: Scalar,
    k: Scalar,
    y: Scalar,
    y_inv: Scalar,
    q: Scalar,
    z_img: Scalar,
    seed: Scalar,
    pub_list: Vec<Scalar>,
    toggle: usize,
) -> ProofResult<(
    R1CSProof,
    Vec<CompressedRistretto>,
    Vec<CompressedRistretto>,
)> {
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(2048, 1);

    let mut transcript = Transcript::new(b"BlindBidProofGadget");

    // 1. Create a prover
    let mut prover = Prover::new(&pc_gens, &mut transcript);

    // 2. Commit high-level variables
    let mut blinding_rng = rand::thread_rng();

    let (commitments, vars): (Vec<_>, Vec<_>) = [d, k, y, y_inv]
        .into_iter()
        .map(|v| prover.commit(*v, Scalar::random(&mut blinding_rng)))
        .unzip();

    let (t_c, t_v): (Vec<_>, Vec<_>) = (0..pub_list.len())
        .map(|x| {
            prover.commit(
                Scalar::from((x == toggle) as u8),
                Scalar::random(&mut thread_rng()),
            )
        })
        .unzip();

    // public list of numbers
    let l_v: Vec<LinearCombination> = pub_list.iter().map(|&x| x.into()).collect::<Vec<_>>();

    // 3. Build a CS
    proof_gadget(
        &mut prover,
        vars[0].into(),
        vars[1].into(),
        vars[3].into(),
        q.into(),
        z_img.into(),
        seed.into(),
        t_v,
        l_v,
    );

    // 4. Make a proof
    let proof = prover.prove(&bp_gens)?;

    Ok((proof, commitments, t_c))
}

pub fn verify(
    proof: R1CSProof,
    commitments: Vec<CompressedRistretto>,
    t_c: Vec<CompressedRistretto>,
    seed: Scalar,
    pub_list: Vec<Scalar>,
    q: Scalar,
    z_img: Scalar,
) -> ProofResult<()> {
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(2048, 1);

    // Verifier logic

    let mut transcript = Transcript::new(b"BlindBidProofGadget");

    // 1. Create a verifier
    let mut verifier = Verifier::new(&mut transcript);

    // 2. Commit high-level variables
    let vars: Vec<_> = commitments.iter().map(|v| verifier.commit(*v)).collect();

    let t_c_v: Vec<Variable> = t_c.iter().map(|v| verifier.commit(*v).into()).collect();

    // public list of numbers
    let l_v: Vec<LinearCombination> = pub_list
        .iter()
        .map(|&x| Scalar::from(x).into())
        .collect::<Vec<_>>();

    // 3. Build a CS
    proof_gadget(
        &mut verifier,
        vars[0].into(),
        vars[1].into(),
        vars[3].into(),
        q.into(),
        z_img.into(),
        seed.into(),
        t_c_v,
        l_v,
    );

    // 4. Verify the proof
    verifier
        .verify(&proof, &pc_gens, &bp_gens)
        .map_err(|_| R1CSError::VerificationError)
}

pub fn proof_gadget<CS: ConstraintSystem>(
    cs: &mut CS,
    d: LinearCombination,
    k: LinearCombination,
    y_inv: LinearCombination,
    q: LinearCombination,
    z_img: LinearCombination,
    seed: LinearCombination,
    toggle: Vec<Variable>, // private: binary list indicating private number is somewhere in list
    items: Vec<LinearCombination>, // public list
) {
    let mut hades = Hash::new();
    // Prove z

    // m = h(k)
    let m = hades.result_gadget(vec![k], cs).unwrap();

    // reset hash
    hades.reset();

    // x = h(d, m)
    let x = hades.result_gadget(vec![d.clone(), m.clone()], cs).unwrap();

    // reset hash
    hades.reset();

    one_of_many_gadget(cs, x.clone(), toggle, items);

    // y = h(seed, x)
    let y = hades.result_gadget(vec![seed.clone(), x], cs).unwrap();

    // reset hash
    hades.reset();

    // z = h(seed, m)
    let z = hades.result_gadget(vec![seed, m], cs).unwrap();

    cs.constrain(z_img - z);

    // Prove Q
    score_gadget(cs, d, y, y_inv, q);
}

fn score_gadget<CS: ConstraintSystem>(
    cs: &mut CS,
    d: LinearCombination,
    y: LinearCombination,
    y_inv: LinearCombination,
    q: LinearCombination,
) {
    let one = Scalar::one();

    // check that Yinv * Y = 1
    let (_, _, one_var) = cs.multiply(y, y_inv.clone());
    cs.constrain(one_var - one);

    // Q = F(d,Y)
    let (_, _, q_var) = cs.multiply(d, y_inv);
    cs.constrain(q - q_var);
}

fn one_of_many_gadget<CS: ConstraintSystem>(
    cs: &mut CS,
    x: LinearCombination,          // private: our item x
    toggle: Vec<Variable>,         // private: binary list indicating it is somewhere in list
    items: Vec<LinearCombination>, // public list
) {
    let toggle_len = toggle.len();

    // ensure every item in toggle is binary
    for i in toggle.iter() {
        boolean_gadget(cs, i.clone().into());
    }

    // toggle_sum[i] = toggle_sum(i-1) + toggle(i)
    let mut toggle_sum: Vec<LinearCombination> = Vec::with_capacity(toggle_len);
    toggle_sum.push(toggle[0].clone().into());
    for i in 1..toggle_len {
        let prev_toggle_sum = toggle_sum[i - 1].clone();
        let curr_toggle = toggle[i].clone();

        toggle_sum.push(prev_toggle_sum + (curr_toggle.clone()));
    }

    // ensure sum of toggles = 1
    for i in 1..toggle_len {
        let prev_toggle_sum = toggle_sum[i - 1].clone();
        let curr_toggle = toggle[i].clone();
        let curr_toggle_sum = toggle_sum[i].clone();

        toggle_sum[i] = toggle_sum[i - 1].clone() + (toggle[i].clone());

        cs.constrain(prev_toggle_sum + (curr_toggle) - (curr_toggle_sum));
    }
    let one: Scalar = Scalar::one();
    let last_item = toggle_sum[toggle_len - 1].clone();
    cs.constrain(last_item - one);

    // now check if item is in list
    // item[i] * toggle[i] = toggle[i] * our item (x)
    for i in 0..toggle_len {
        let (_, _, left) = cs.multiply(items[i].clone(), toggle[i].clone().into());
        let (_, _, right) = cs.multiply(toggle[i].clone().into(), x.clone());
        cs.constrain(left - right);
    }
}

fn boolean_gadget<CS: ConstraintSystem>(cs: &mut CS, a1: LinearCombination) {
    // a *(1-a) = 0
    let a = a1.clone();
    let one: LinearCombination = Scalar::one().into();
    let (_, _, c_var) = cs.multiply(a, one - a1);
    cs.constrain(c_var.into());
}
