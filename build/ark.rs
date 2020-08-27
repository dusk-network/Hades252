// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.”
use super::BlsScalar;
use sha2::{Digest, Sha512};
use std::fs;
use std::io::Write;

const CONSTANTS: usize = 960;

fn constants() -> [BlsScalar; CONSTANTS] {
    let mut cnst = [BlsScalar::zero(); CONSTANTS];
    let mut p = BlsScalar::one();
    let mut bytes = b"poseidon-for-plonk".to_vec();

    cnst.iter_mut().for_each(|c| {
        let mut hasher = Sha512::new();
        hasher.input(bytes.as_slice());
        bytes = hasher.result().to_vec();

        let mut v = [0x00u8; 64];
        v.copy_from_slice(&bytes[0..64]);

        *c = BlsScalar::from_bytes_wide(&v) + p;
        p = *c;
    });

    cnst
}

pub(crate) fn write_to(filename: &str) -> std::io::Result<()> {
    let mut buf: Vec<u8> = vec![];

    constants().iter().for_each(|c| {
        c.internal_repr()
            .iter()
            .for_each(|r| buf.extend_from_slice(&(*r).to_le_bytes()));
    });

    let mut file = fs::File::create(filename)?;
    file.write_all(&buf)?;
    Ok(())
}
