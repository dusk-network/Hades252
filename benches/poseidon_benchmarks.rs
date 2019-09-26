#[macro_use]
extern crate criterion;
extern crate hades252;
extern crate bulletproofs;

use criterion::Criterion;
use hades252::hash::Hash;
use curve25519_dalek::scalar::Scalar;

/// Since we can only bench a single function, we make a function
/// that instantiates the hash, inputs the vec of `Scalar` and then
/// calls `result()`.
fn inst_plus_digest() -> () {
    let mut hasher = Hash::new();
    let input = vec![Scalar::one(); 7];
    hasher.inputs(input).unwrap();
    hasher.result();
}

pub fn hasher_creation(c: &mut Criterion) {
    c.bench_function("Non-optimized hasher creation", |b| b.iter(|| Hash::new()));
}

/// Since this implementation, needs to instanciate the hasher,
/// we need to bench a function that does it. 
/// Thats why we bench `inst_plus_digest`.
pub fn hash_scalar_vec(c: &mut Criterion) {
    let mut hasher = Hash::new();
    hasher.input(Scalar::one()).unwrap();

    c.bench_function("Non-optimized single Scalar hash", |b| b.iter(|| 
        inst_plus_digest())
    );
}

criterion_group!(benches, 
    hasher_creation, 
    hash_scalar_vec
);
criterion_main!(benches);