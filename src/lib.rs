// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![deny(missing_docs)]
#![cfg_attr(feature = "std", feature(external_doc))]
#![cfg_attr(feature = "std", doc(include = "../README.md"))]
#![cfg_attr(not(feature = "std"), no_std)]

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

#[cfg(feature = "std")]
pub use strategies::GadgetStrategy;
pub use strategies::{ScalarStrategy, Strategy};
