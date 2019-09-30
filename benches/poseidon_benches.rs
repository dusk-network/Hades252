#[macro_use]
extern crate criterion;
extern crate hades252;

use criterion::black_box;
use criterion::Criterion;
use rand::thread_rng;

use bulletproofs::r1cs::{LinearCombination, Verifier};
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

use hades252::linear_combination;
use hades252::scalar;

/// This function allows us to bench the whole process of
/// digesting a vec of `Scalar`. We put together the process
/// of creating the scalar array + digesting the info to be able
/// to compare the results against the unoptimized version.
#[inline]
fn digest_one() -> () {
    let s = Scalar::random(&mut thread_rng());
    scalar::hash(&[s]).unwrap();
}

#[inline]
fn digest(n: usize) -> () {
    let s: Vec<Scalar> = (0..n).map(|_| Scalar::random(&mut thread_rng())).collect();
    scalar::hash(&s).unwrap();
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
