//#![feature(trait_alias)]
//#![feature(external_doc)]
//#![deny(missing_docs)]
//#![doc(include = "../README.md")]
//#![feature(test)]
mod mds_matrix;
mod round_constants;

pub mod strategies;

/// Total ammount of full rounds that will be applied.
/// This is expressed as `RF` in the paper.
pub const TOTAL_FULL_ROUNDS: usize = 8;

/// Total ammount of partial rounds that will be applied.
/// This is expressed as `Rp` in the paper.
pub const PARTIAL_ROUNDS: usize = 59;

/// Maximum input width for the rounds
pub const WIDTH: usize = 5;

pub use dusk_bls12_381::Scalar as BlsScalar;

//pub use algebra::curves::bls12_381::Bls12_381;
//pub use algebra::curves::jubjub::JubJubProjective;
//pub use algebra::fields::jubjub::fq::Fq;
pub use strategies::{GadgetStrategy, ScalarStrategy, Strategy};
