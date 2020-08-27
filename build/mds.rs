// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.”
use super::BlsScalar;
use std::fs;
use std::io::Write;

const WIDTH: usize = 5;

fn mds() -> [[BlsScalar; WIDTH]; WIDTH] {
    let mut matrix = [[BlsScalar::zero(); WIDTH]; WIDTH];
    let mut xs = [BlsScalar::zero(); WIDTH];
    let mut ys = [BlsScalar::zero(); WIDTH];

    // Generate x and y values deterministically for the cauchy matrix
    // where x[i] != y[i] to allow the values to be inverted
    // and there are no duplicates in the x vector or y vector, so that the determinant is always non-zero
    // [a b]
    // [c d]
    // det(M) = (ad - bc) ; if a == b and c == d => det(M) =0
    // For an MDS matrix, every possible mxm submatrix, must have det(M) != 0
    (0..WIDTH).for_each(|i| {
        xs[i] = BlsScalar::from(i as u64);
        ys[i] = BlsScalar::from((i + WIDTH) as u64);
    });

    let mut m = 0;
    (0..WIDTH).for_each(|i| {
        (0..WIDTH).for_each(|j| {
            matrix[m][j] = (xs[i] + ys[j]).invert().unwrap();
        });
        m += 1;
    });

    matrix
}

pub(crate) fn write_to(filename: &str) -> std::io::Result<()> {
    let mut buf: Vec<u8> = vec![];

    mds().iter().for_each(|row| {
        row.iter().for_each(|c| {
            c.internal_repr()
                .iter()
                .for_each(|r| buf.extend_from_slice(&(*r).to_le_bytes()));
        });
    });

    let mut file = fs::File::create(filename)?;
    file.write_all(&buf)?;
    Ok(())
}
