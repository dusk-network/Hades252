use super::Strategy;
use crate::{mds_matrix::MDS_MATRIX, BlsScalar, WIDTH};

/// Implements a Hades252 strategy for `BlsScalar` as input values.
#[derive(Default)]
pub struct ScalarStrategy {}

impl ScalarStrategy {
    /// Constructs a new `ScalarStrategy`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Strategy<BlsScalar> for ScalarStrategy {
    fn quintic_s_box(&mut self, value: &mut BlsScalar) {
        let s = *value;

        *value = s * s * s * s * s;
    }

    fn mul_matrix(&mut self, values: &mut [BlsScalar]) {
        let mut result = [BlsScalar::zero(); WIDTH];

        for j in 0..WIDTH {
            for k in 0..WIDTH {
                result[k] += MDS_MATRIX[k][j] * values[j];
            }
        }

        values.copy_from_slice(&result);
    }

    fn add_round_key<'b, I>(&mut self, constants: &mut I, words: &mut [BlsScalar])
    where
        I: Iterator<Item = &'b BlsScalar>,
    {
        words.iter_mut().for_each(|w| {
            *w += constants.next().unwrap_or(&BlsScalar::one());
        });
    }

    /// Perform a slice strategy
    fn poseidon_slice(&mut self, data: &[BlsScalar]) -> BlsScalar {
        let mut perm = [BlsScalar::zero(); WIDTH];

        data.chunks(WIDTH - 2).fold(BlsScalar::zero(), |r, chunk| {
            perm[0] = BlsScalar::from(chunk.len() as u64);
            perm[1] = r;

            chunk
                .iter()
                .zip(perm.iter_mut().skip(2))
                .for_each(|(c, p)| *p = *c);

            self.poseidon(&mut perm)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{BlsScalar, ScalarStrategy, Strategy, WIDTH};

    fn perm(values: &mut [BlsScalar]) {
        let mut strategy = ScalarStrategy::new();
        strategy.perm(values);
    }

    #[test]
    fn hades_det() {
        let mut x = [BlsScalar::from(17u64); WIDTH];
        let mut y = [BlsScalar::from(17u64); WIDTH];
        let mut z = [BlsScalar::from(19u64); WIDTH];

        perm(&mut x);
        perm(&mut y);
        perm(&mut z);

        assert_eq!(x, y);
        assert_ne!(x, z);
    }
}
