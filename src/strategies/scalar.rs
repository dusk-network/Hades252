use crate::{mds_matrix::MDS_MATRIX, Fq, WIDTH};

use super::Strategy;

use num_traits::{One, Zero};

/// Implements a Hades252 strategy for `Fq` as input values.
#[derive(Default)]
pub struct ScalarStrategy {}

impl ScalarStrategy {
    /// Constructs a new `ScalarStrategy`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Strategy<Fq> for ScalarStrategy {
    fn quintic_s_box(&mut self, value: &mut Fq) {
        let s = *value;

        *value = s * s * s * s * s;
    }

    fn mul_matrix(&mut self, values: &mut [Fq]) {
        let mut result = [Fq::zero(); WIDTH];

        for j in 0..WIDTH {
            for k in 0..WIDTH {
                result[k] += MDS_MATRIX[k][j] * values[j];
            }
        }

        values.copy_from_slice(&result);
    }

    fn add_round_key<'b, I>(&mut self, constants: &mut I, words: &mut [Fq])
    where
        I: Iterator<Item = &'b Fq>,
    {
        words.iter_mut().for_each(|w| {
            *w += constants.next().unwrap_or(&Fq::one());
        });
    }

    /// Perform a slice strategy
    fn poseidon_slice(&mut self, data: &[Fq]) -> Fq {
        let mut perm = [Fq::zero(); WIDTH];

        data.chunks(WIDTH - 2).fold(Fq::zero(), |r, chunk| {
            perm[0] = Fq::from(chunk.len() as u64);
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
    use crate::{Fq, ScalarStrategy, Strategy, WIDTH};

    fn perm(values: &mut [Fq]) {
        let mut strategy = ScalarStrategy::new();
        strategy.perm(values);
    }

    #[test]
    fn hades_det() {
        let mut x = [Fq::from(17u64); WIDTH];
        let mut y = [Fq::from(17u64); WIDTH];
        let mut z = [Fq::from(19u64); WIDTH];

        perm(&mut x);
        perm(&mut y);
        perm(&mut z);

        assert_eq!(x, y);
        assert_ne!(x, z);
    }
}
