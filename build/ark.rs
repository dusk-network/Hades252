use super::{RawScalar, Scalar};
use std::{fs::File, io::prelude::*};

use sha2::{Digest, Sha512};

const CONSTANTS: usize = 960;

fn constants() -> [[u8; 32]; CONSTANTS] {
    // TODO - Review constants generation
    let mut cnst = [[0x00u8; 32]; CONSTANTS];
    let mut p = Scalar::one();
    let mut bytes = b"poseidon-for-plonk".to_vec();
    let two_inv = Scalar::from(2u64).invert().unwrap();

    (0..CONSTANTS).for_each(|i| {
        let mut hasher = Sha512::new();
        hasher.input(bytes.as_slice());
        bytes = hasher.result().to_vec();

        let mut v = [0x00u8; 64];
        v.copy_from_slice(&bytes[0..64]);

        let s = Scalar::from_bytes_wide(&v) + p * two_inv;
        cnst[i] = unsafe { std::mem::transmute(RawScalar::from(s).0) };
        p = s;
    });

    cnst
}

pub(crate) fn write_to(filename: &str) -> std::io::Result<()> {
    let mut buf: Vec<u8> = vec![];

    constants()
        .iter()
        .for_each(|b| buf.extend_from_slice(&b[..]));

    let mut file = File::create(filename)?;
    file.write_all(&buf)?;
    Ok(())
}
