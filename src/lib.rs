//#![feature(trait_alias)]
//#![feature(external_doc)]
//!
#![deny(missing_docs)]
#![cfg_attr(feature = "nightly_docs", feature(external_doc))]
#![cfg_attr(feature = "nightly_docs", doc(include = "../README.md"))]

mod mds_matrix;
mod round_constants;

/// Strategies implemented for the Hades252 algorithm.
pub mod strategies;

/// Total ammount of full rounds that will be applied.
/// This is expressed as `RF` in the paper.
pub const TOTAL_FULL_ROUNDS: usize = 8;

/// Total ammount of partial rounds that will be applied.
/// This is expressed as `Rp` in the paper.
pub const PARTIAL_ROUNDS: usize = 59;

/// Maximum input width for the rounds
pub const WIDTH: usize = 5;

pub use strategies::{GadgetStrategy, ScalarStrategy, Strategy};
