use super::Scalar;
use std::{cmp, fs::File, io::prelude::*};

use algebra::biginteger::BigInteger256;
use num_traits::{One, Zero};
use sha2::{Digest, Sha512};

const CONSTANTS: usize = 960;

fn slice_to_u64(bytes: &[u8]) -> u64 {
    let mut s = [0x00u8; 8];
    let chunk = cmp::min(bytes.len(), 8);

    (&mut s[0..chunk]).copy_from_slice(&bytes[0..chunk]);

    u64::from_be_bytes(s)
}

fn constants() -> [Scalar; 960] {
    // TODO - Review constants generation
    let mut cnst = [Scalar::zero(); 960];
    let mut p = Scalar::one();
    let mut bytes = b"poseidon-for-plonk".to_vec();
    let two = Scalar::from(2u64);

    (0..CONSTANTS).for_each(|i| {
        let mut hasher = Sha512::new();
        hasher.input(bytes.as_slice());
        bytes = hasher.result().to_vec();

        let mut v = [0x00u8; 64];
        v.copy_from_slice(&bytes[0..64]);

        let a = slice_to_u64(&bytes[0..]);
        let b = slice_to_u64(&bytes[8..]);
        let c = slice_to_u64(&bytes[16..]);
        let d = slice_to_u64(&bytes[24..]);
        let s = Scalar::from(BigInteger256([a, b, c, d]));

        cnst[i] = s + p / two;
        p = cnst[i];
    });

    cnst
}

pub(crate) fn write_to(filename: &str) -> std::io::Result<()> {
    let mut buf: Vec<u8> = vec![];

    constants().iter().for_each(|c| {
        for n in (c.0).0.iter() {
            buf.extend_from_slice(&n.to_le_bytes());
        }
    });

    let mut file = File::create(filename)?;
    file.write_all(&buf)?;
    Ok(())
}
