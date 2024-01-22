// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

/// Build file for the dusk-hades crate.
///
/// The purpose of this file is to add a deprecation warning at compile time.

#[deprecated(
    note = "This crate is not in active development anymore, use 'dusk-poseidon' instead."
)]
const DEPRECATED: bool = true;

fn main() {
    // Ensure we run the build script again when only 'build.rs' has changed
    println!("cargo:rerun-if-changed=build.rs");

    #[allow(deprecated)]
    let _dusk_hades = DEPRECATED;
}
