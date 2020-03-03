use super::Scalar;
use std::{fs::File, io::prelude::*};

use num_traits::{One, Zero};

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
            matrix[m][j] = Scalar::one() / (xs[i] + ys[j]);
        });
        m += 1;
    });

    matrix
}

pub(crate) fn write_to(filename: &str) -> std::io::Result<()> {
    let mut buf: Vec<u8> = vec![];

    mds().iter().for_each(|row| {
        row.iter().for_each(|c| {
            for n in (c.0).0.iter() {
                buf.extend_from_slice(&n.to_le_bytes());
            }
        });
    });

    let mut file = File::create(filename)?;
    file.write_all(&buf)?;
    Ok(())
}
