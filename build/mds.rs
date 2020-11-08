// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use super::Scalar;
use std::fs;
use std::io::Write;

const WIDTH: usize = 5;

fn mds() -> [[Scalar; WIDTH]; WIDTH] {
    let mut matrix = [[Scalar::zero(); WIDTH]; WIDTH];
    let mut xs = [Scalar::zero(); WIDTH];
    let mut ys = [Scalar::zero(); WIDTH];

    // Generate x and y values deterministically for the cauchy matrix
    // where x[i] != y[i] to allow the values to be inverted
    // and there are no duplicates in the x vector or y vector, so that the determinant is always non-zero
    // [a b]
    // [c d]
    // det(M) = (ad - bc) ; if a == b and c == d => det(M) =0
    // For an MDS matrix, every possible mxm submatrix, must have det(M) != 0
    (0..WIDTH).for_each(|i| {
        xs[i] = Scalar::from(i as u64);
        ys[i] = Scalar::from((i + WIDTH) as u64);
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
