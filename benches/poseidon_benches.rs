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


mod scalar_benches {
    use super::*;

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
}

mod lc_benches {
    use super::*;

    /// This function allows us to bench the whole process of
    /// digesting a vec of `LinearCombination`. We put together the process
    /// of creating the linearcombination array + digesting the info to be able
    /// to compare the results against the unoptimized version.
    #[inline]
    fn digest_one() -> () {
        let s = LinearCombination::from(Scalar::random(&mut thread_rng()));

        // The creation of the `Verifier` does not affect at the benchmark timings.
        // It takes aproximately: `625.21 ns`.
        let mut verifier_transcript = Transcript::new(b"");
        let mut verifier = Verifier::new(&mut verifier_transcript);

        linear_combination::hash(&mut verifier, &[s]).unwrap();
    }

    #[inline]
    fn digest(n: usize) -> () {
        let s: Vec<LinearCombination> = (0..n).map(|_| LinearCombination::from(Scalar::random(&mut thread_rng()))).collect();
        
        // The creation of the `Verifier` does not affect at the benchmark timings.
        // It takes aproximately: `625.21 ns`.
        let mut verifier_transcript = Transcript::new(b"");
        let mut verifier = Verifier::new(&mut verifier_transcript);

        linear_combination::hash(&mut verifier, s.as_slice()).unwrap();
    }

    pub fn hash_one_lc(c: &mut Criterion) {
        c.bench_function("Hashing single `LinearCombination`", |b| b.iter(|| digest_one()));
    }

    pub fn hash_two_lcs(c: &mut Criterion) {
        c.bench_function("Hashing two `LinearCombination`", |b| b.iter(|| digest(black_box(2))));
    }

    pub fn hash_four_lcs(c: &mut Criterion) {
        c.bench_function("Hashing four `LinearCombination`", |b| b.iter(|| digest(black_box(4))));
    }
}

criterion_group!(
    benches,
    scalar_benches::hash_one_scalar,
    scalar_benches::hash_two_scalars,
    scalar_benches::hash_four_scalars,
    lc_benches::hash_one_lc,
    lc_benches::hash_two_lcs,
    lc_benches::hash_four_lcs
);

criterion_main!(benches);
