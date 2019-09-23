#![allow(non_snake_case)]

use bulletproofs::r1cs::LinearCombination;
use curve25519_dalek::scalar::Scalar;

pub struct MDSMatrix {
    matrix: Vec<Vec<Scalar>>,
}

impl MDSMatrix {
    pub fn generate(t: usize) -> Self {
        let offset = 0;
        MDSMatrix {
            matrix: MDSMatrix::with_offset(t, offset),
        }
    }

    pub fn matrix(&self) -> &Vec<Vec<Scalar>> {
        &self.matrix
    }

    /// generates a `t` by `t` MDS matrix with elements in GF(p)  
    /// Note that this will not match any reference implementation because
    /// curve25519 uses Little-Endian format, while other implementations use big-Endian
    pub fn with_offset(t: usize, start: usize) -> Vec<Vec<Scalar>> {
        let mut matrix: Vec<Vec<Scalar>> = Vec::with_capacity(t);
        let mut xs: Vec<Scalar> = Vec::with_capacity(t);
        let mut ys: Vec<Scalar> = Vec::with_capacity(t);

        // Generate x and y values deterministically for the cauchy matrix
        // where x[i] != y[i] to allow the values to be inverted
        // and there are no duplicates in the x vector or y vector, so that the determinant is always non-zero
        // [a b]
        // [c d]
        // det(M) = (ad - bc) ; if a == b and c == d => det(M) =0
        // For an MDS matrix, every possible mxm submatrix, must have det(M) != 0
        for i in 0..t {
            let x = Scalar::from((i + start) as u64);
            let y = Scalar::from((i + start + t) as u64);
            xs.push(x);
            ys.push(y);
        }

        for i in 0..t {
            let mut row: Vec<Scalar> = Vec::with_capacity(t);
            for j in 0..t {
                // Generate the entry at (i,j)
                let entry = (xs[i] + ys[j]).invert();
                row.insert(j, entry);
            }
            matrix.push(row);
        }
        matrix
    }

    // Matrix-Vector multiplication; multiply the MDS matrix by the given vector
    pub fn mul_vector(&self, b: &[Scalar]) -> Vec<Scalar> {
        self.matrix.iter().map(|row| dot_product(row, b)).collect()
    }

    pub fn constrain_mul_vector(&self, b: Vec<LinearCombination>) -> Vec<LinearCombination> {
        self.matrix
            .iter()
            .map(|row| constrain_dot_product(row, b.clone()))
            .collect()
    }
}

fn dot_product(a: &[Scalar], b: &[Scalar]) -> Scalar {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn constrain_dot_product(a: &[Scalar], b: Vec<LinearCombination>) -> LinearCombination {
    let l_cs: Vec<LinearCombination> = a
        .iter()
        .zip(b.iter())
        .map(|(a_i, b_i)| a_i.clone() * b_i.clone())
        .collect();

    let mut sum: LinearCombination = Scalar::zero().into();

    for l_c in l_cs {
        sum = sum + l_c;
    }

    sum.simplify()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mds() {
        let t = 6;

        let mds = MDSMatrix::generate(t);
        assert_eq!(t, mds.matrix.len());

        for column in mds.matrix {
            assert_eq!(t, column.len());
        }
    }
}
