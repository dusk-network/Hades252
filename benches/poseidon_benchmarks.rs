#[macro_use]
extern crate criterion;
extern crate hades252;
extern crate bulletproofs;

use criterion::Criterion;
use hades252::hash::Hash;
use curve25519_dalek::scalar::Scalar;

pub fn hasher_creation(c: &mut Criterion) {
    c.bench_function("Non-optimized hasher creation", |b| b.iter(|| Hash::new()));
}

pub fn hasher_scalar_round(c: &mut Criterion) {
    let mut hasher = Hash::new();
    hasher.input(Scalar::one()).unwrap();

    c.bench_function("Non-optimized single Scalar hash", |b| b.iter(|| 
        hasher.result())
    );
}

pub fn hasher_vec_round(c: &mut Criterion) {
    let mut hasher = Hash::new(); 
    let vec_inp = vec!(Scalar::one(), Scalar::one(), Scalar::one(), Scalar::one(), Scalar::one());
    hasher.inputs(vec_inp).unwrap();

    c.bench_function("Non-optimized single Scalar hash", |b| b.iter(|| 
        hasher.result())
    );
}

pub fn hasher_slice_round(c: &mut Criterion) {
    let mut hasher = Hash::new(); 
    let slice: [u8; 32] = [194, 24, 45, 158, 220, 161, 164, 1, 231, 42, 46, 200, 184, 98, 31, 166, 153, 153, 153, 153, 153, 153, 153, 153, 153, 153, 153, 153, 153, 153, 153, 9];
    hasher.input_bytes(&slice).unwrap();

    c.bench_function("Non-optimized single Scalar hash", |b| b.iter(|| 
        hasher.result())
    );
}


criterion_group!(benches, hasher_creation, 
    hasher_scalar_round, 
    hasher_vec_round,
    hasher_slice_round,
);
criterion_main!(benches);