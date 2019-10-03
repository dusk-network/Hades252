//! # Hades252
//! 
//! Implementation of Hades252 over the Ristretto Scalar field.
//! 
//! *Unstable* : No guarantees can be made regarding the API stability.
//! 
//! ## Parameters
//! 
//! - p = `2^252 + 27742317777372353535851937790883648493`
//! 
//! - Security level is 126 bits
//! 
//! - width = 9
//! 
//! - Number of full rounds = 8 . There are four full rounds at the beginning and four full rounds at the end,
//! where each full round has `WIDTH` quintic S-Boxes.
//! 
//! - Merkle tree arity : 8 to 1
//! 
//! - Number of partial rounds = 59, where each partial round has one quintic S-Box and (width-1) identity functions.
//! 
//! - Number of round constants = 960
//! 
//! ## Example outside R1CS
//! ```
//! use hades252::scalar::hash;
//! use curve25519_dalek::scalar::Scalar;
//! 
//! // Generate the inputs that will be hashed.
//! // Since the `WIDTH` is set to `9` we can use `8`
//! // inputs as much.
//! // If we use less, the function will take care of padding them
//! // propperly.
//! let input = [Scalar::one(); 7];
//! 
//! // Hash the data and get the resulting `Scalar`.
//! let digest_res = hash(&input).unwrap();
//! ```
//! 
//! ## Example inside R1CS
//! ```
//! use hades252::linear_combination::hash;
//! use curve25519_dalek::scalar::Scalar;
//! use bulletproofs::r1cs::{LinearCombination, Verifier};
//! use merlin::Transcript;
//! 
//! // Generate the inputs that will be hashed.
//! // Since the `WIDTH` is set to `9` we can use `8`
//! // inputs as much.
//! // If we use less, the function will take care of padding them
//! // propperly.
//! let inp = LinearCombination::from(Scalar::one());
//! let mut lc_one: Vec<LinearCombination> = vec![];
//! for _ in 0..7 {
//!     lc_one.push(inp.clone());
//! };
//! 
//! // Generate the Verifier (Prover is not generated)
//! // since this is just a demo.
//! let mut verifier_transcript = Transcript::new(b"");
//! let mut verifier = Verifier::new(&mut verifier_transcript);
//! 
//! // Hash the data and get the resulting `LinearCombination`.
//! let digest_res = hash(&mut verifier, &lc_one).unwrap();
//! ```
//! 
//! ## Deviations
//! 
//! - Round constants for the full rounds are generated following: https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py
//! They are then mapped onto `Scalar`s in the Ristretto scalar field.
//! 
//! - The MDS matrix is a cauchy matrix, the method used to generate it, is noted in section "Concrete Instantiations Poseidon and Starkad"
//! 
//! ## Reference
//! 
//! https://eprint.iacr.org/2019/458.pdf

#![feature(test)]
/// Declaration of the errors that the crate can produce.
pub mod errors;
mod mds_matrix;
/// Scalar backend for `Outside-R1CS` hashing operations.
pub mod scalar;
/// LinearCombination backend for `Inside-R1CS` hashing operations.
pub mod linear_combination;
mod round_constants;
