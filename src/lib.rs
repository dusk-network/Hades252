#![feature(trait_alias)]
#![feature(external_doc)]
#![deny(missing_docs)]
#![doc(include = "../README.md")]
#![feature(test)]
/// Declaration of the errors that the crate can produce.
mod mds_matrix;
mod round_constants;
pub mod strategies;

mod settings;
pub use settings::*;
