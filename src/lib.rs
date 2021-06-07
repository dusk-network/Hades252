// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![deny(missing_docs)]
#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

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

#[cfg(feature = "alloc")]
pub use strategies::GadgetStrategy;
pub use strategies::{ScalarStrategy, Strategy};

pub(crate) const fn u64_from_buffer<const N: usize>(buf: &[u8; N], i: usize) -> u64 {
    u64::from_le_bytes([
        buf[i],
        buf[i + 1],
        buf[i + 2],
        buf[i + 3],
        buf[i + 4],
        buf[i + 5],
        buf[i + 6],
        buf[i + 7],
    ])
}
