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

criterion_group!(benches, hasher_creation);
criterion_main!(benches);