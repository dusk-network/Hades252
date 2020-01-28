use crate::settings_width::get_width;
use std::fs::File;
use std::io::prelude::*;

use curve25519_dalek::scalar::Scalar;

pub(crate) fn write_to(filename: &str) -> std::io::Result<()> {
    let width = get_width();
    let size = width * width;
    let mut buf: Vec<u8> = Vec::with_capacity(size << 5);
    let mut xs: Vec<Scalar> = Vec::with_capacity(width);
    let mut ys: Vec<Scalar> = Vec::with_capacity(width);

    // Generate x and y values deterministically for the cauchy matrix
    // where x[i] != y[i] to allow the values to be inverted
    // and there are no duplicates in the x vector or y vector, so that the
    // determinant is always non-zero
    // [a b]
    // [c d]
    // det(M) = (ad - bc) ; if a == b and c == d => det(M) =0
    // For an MDS matrix, every possible mxm submatrix, must have det(M) != 0
    for i in 0..width {
        let x = Scalar::from(i as u64);
        let y = Scalar::from((i + width) as u64);
        xs.push(x);
        ys.push(y);
    }

    for i in 0..width {
        for j in 0..width {
            // Generate the entry at (i,j)
            let entry = (xs[i] + ys[j]).invert();
            buf.extend_from_slice(&entry.to_bytes())
        }
    }

    let mut file = File::create(filename)?;
    file.write_all(&buf)?;

    Ok(())
}
