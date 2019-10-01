#[macro_use]
extern crate criterion;
extern crate bulletproofs;
extern crate hades252;

use criterion::black_box;
use criterion::Criterion;
use rand::thread_rng;

use bulletproofs::r1cs::{LinearCombination, Verifier};
use merlin::Transcript;
use curve25519_dalek::scalar::Scalar;
use hades252::hash::Hash;

mod scalar_benches {
    use super::*;
    
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
}

mod lc_benches {
    use super::*;

    /// This function allows us to bench the whole process of
    /// digesting a vec of `LinearCombination`. We put together the process
    /// of creating the linearcombination array + digesting the info to be able
    /// to compare the results against the unoptimized version.
    #[inline]
    fn digest_one() -> () {
        let mut hasher = Hash::new();
        let s = LinearCombination::from(Scalar::random(&mut thread_rng()));

        let mut verifier_transcript = Transcript::new(b"");
        let mut verifier = Verifier::new(&mut verifier_transcript);

        hasher.input_lc(s).unwrap();
        hasher.result_gadget(&mut verifier).unwrap();
    }

    #[inline]
    fn digest(n: usize) -> () {
        let mut hasher = Hash::new();
        let lcs: Vec<LinearCombination> = (0..n).map(|_| LinearCombination::from(Scalar::random(&mut thread_rng()))).collect();
        for lc in lcs {
            hasher.input_lc(lc).unwrap()
        };
        
        let mut verifier_transcript = Transcript::new(b"");
        let mut verifier = Verifier::new(&mut verifier_transcript);

        hasher.result_gadget(&mut verifier).unwrap();
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
