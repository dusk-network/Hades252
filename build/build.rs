// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.”
mod ark;
mod mds;

pub use dusk_plonk::bls12_381::Scalar as BlsScalar;

fn main() -> std::io::Result<()> {
    ark::write_to("assets/ark.bin")?;
    mds::write_to("assets/mds.bin")?;
    Ok(())
}
