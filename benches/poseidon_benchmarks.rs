#[macro_use]
extern crate criterion;
extern crate bulletproofs;
extern crate hades252;

use criterion::black_box;
use criterion::Criterion;
use rand::thread_rng;

use bulletproofs::r1cs::LinearCombination;
use curve25519_dalek::scalar::Scalar;
use hades252::hash::Hash;

/// This function allows us to bench the whole process of
/// digesting a vec of `Scalar`. Since the hasher instantiation
/// time matters here, and is optimized or even non-existing on other
/// implementations, that makes us bench the whole process instead of
/// just benching the `hasher.result()` fn.
#[inline]
fn digest_one() -> () {
    let mut hasher = Hash::new();
    let s = Scalar::random(&mut thread_rng());
    hasher.input(s).unwrap();
    hasher.result();
}

#[inline]
fn digest(n: usize) -> () {
    let mut hasher = Hash::new();
    let s: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut thread_rng())).collect();
    hasher.inputs(s).unwrap();
    hasher.result();
}

pub fn hash_one_scalar(c: &mut Criterion) {
    c.bench_function("Hashing single `Scalar`", |b| b.iter(|| digest_one()));
}

pub fn hash_two_scalars(c: &mut Criterion) {
    c.bench_function("Hashing two `Scalar`", |b| b.iter(|| digest(black_box(2))));
}

pub fn hash_four_scalars(c: &mut Criterion) {
    c.bench_function("Hashing four `Scalar`", |b| b.iter(|| digest(black_box(4))));
}

criterion_group!(
    benches,
    hash_one_scalar,
    hash_two_scalars,
    hash_four_scalars
);

criterion_main!(benches);
