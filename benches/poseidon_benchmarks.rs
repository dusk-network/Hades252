#[macro_use]
extern crate criterion;
extern crate hades252;
extern crate bulletproofs;

use criterion::Criterion;
use hades252::hash::Hash;
use curve25519_dalek::scalar::Scalar;
use bulletproofs::r1cs::LinearCombination;

/// This function allows us to bench the whole process of 
/// digesting a vec of `Scalar`. Since the hasher instantiation
/// time matters here, and is optimized or even non-existing on other
/// implementations, that makes us bench the whole process instead of
/// just benching the `hasher.result()` fn.
fn inst_plus_digest() -> () {
    let mut hasher = Hash::new();
    let input = vec![Scalar::one(); 7];
    hasher.inputs(input).unwrap();
    hasher.result();
}

/// This function allows us to bench the whole process of 
/// digesting a vec of `LinearCombination`. Since the hasher instantiation
/// time matters here, and is optimized or even non-existing on other
/// implementations, that makes us bench the whole process instead of
/// just benching the `hasher.result()` fn.
fn inst_plus_digest_lc() -> () {
    let mut hasher = Hash::new();
    let lc_one = LinearCombination::from(Scalar::one());
    for _ in 0..7 {
        hasher.input_lc(lc_one.clone()).unwrap();
    }
    hasher.result();
}

pub fn hasher_creation(c: &mut Criterion) {
    c.bench_function("Non-optimized hasher instantiation", |b| b.iter(|| Hash::new()));
}

pub fn hash_scalar_vec(c: &mut Criterion) {
    c.bench_function("Non-optimized Instantiation + Vec<Scalar> hash", |b| b.iter(|| 
        inst_plus_digest())
    );
}

pub fn hash_lc_vec(c: &mut Criterion) {
    c.bench_function("Non-optimized Instantiation + Vec<LinerCombination> hash", |b| b.iter(|| 
        inst_plus_digest_lc())
    );
}

criterion_group!(benches, 
    hasher_creation, 
    hash_scalar_vec,
    hash_lc_vec
);
criterion_main!(benches);