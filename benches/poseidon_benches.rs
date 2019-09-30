#[macro_use]
extern crate criterion;
extern crate hades252;

use criterion::Criterion;

use curve25519_dalek::scalar::Scalar;
use bulletproofs::r1cs::{LinearCombination, Verifier};
use merlin::Transcript;

use hades252::scalar;
use hades252::linear_combination;


/// This function allows us to bench the whole process of 
/// digesting a vec of `Scalar`. We put together the process
/// of creating the scalar array + digesting the info to be able
/// to compare the results against the unoptimized version.
fn inst_plus_digest() -> () {
    let input = [Scalar::one(); 7];
    scalar::hash(&input).unwrap();
}

/// This function allows us to bench the whole process of 
/// digesting a vec of `LinearCombination`. We put together the process
/// of creating the scalar array + digesting the info to be able
/// to compare the results against the unoptimized version.
fn inst_plus_digest_lc() -> () {
    let inp = LinearCombination::from(Scalar::one());
    let mut lc_one: Vec<LinearCombination> = vec![];
    for _ in 0..7 {
        lc_one.push(inp.clone());
    };

    let mut verifier_transcript = Transcript::new(b"");
    let mut verifier = Verifier::new(&mut verifier_transcript);
    linear_combination::hash(&mut verifier, &lc_one).unwrap();
}

//----------------- Benches -----------------//
pub fn hash_scalar_vec(c: &mut Criterion) {
    c.bench_function("Optimized Instantiation + Vec<Scalar> hash", |b| b.iter(|| 
        inst_plus_digest())
    );
}

pub fn hash_lc_vec(c: &mut Criterion) {
    c.bench_function("Optimized Instantiation + Vec<LinerCombination> hash", |b| b.iter(|| 
        inst_plus_digest_lc())
    );
}


criterion_group!(benches,  
    hash_scalar_vec
);
criterion_main!(benches);