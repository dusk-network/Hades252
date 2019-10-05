use crate::errors::PermError;
use crate::scalar::{self, Scalar};
use crate::{MERKLE_ARITY, MERKLE_INNER_WIDTH, MERKLE_WIDTH, WIDTH};

pub fn hash(data: &[Option<Scalar>]) -> Result<Scalar, PermError> {
    if data.len() > MERKLE_ARITY {
        return Err(PermError::MerkleWidthOverflow);
    }

    // Normalize the input
    let mut leaves = [Scalar::zero(); WIDTH];
    let mut bitflag = 0u64;

    data.iter().enumerate().for_each(|(i, scalar)| {
        if let Some(s) = scalar {
            leaves[i + 1] = *s;
            bitflag |= 1u64 << MERKLE_ARITY - i - 1;
        }
    });

    leaves[0] = Scalar::from(bitflag);
    scalar::perm(leaves.to_vec()).map(|p| p[1])
}

pub fn root(data: &[Option<Scalar>]) -> Result<Scalar, PermError> {
    if data.len() > MERKLE_WIDTH {
        return Err(PermError::MerkleWidthOverflow);
    }

    let mut row = input_to_merkle_row(data);
    let mut merkle = MERKLE_WIDTH / MERKLE_ARITY;
    let mut raw_index;
    let full_flags = Scalar::from((1u64 << MERKLE_ARITY) - 1);

    while merkle > 0 {
        raw_index = 0;

        for j in 0..merkle {
            raw_index += 1;

            // Skip the bitflags position
            if raw_index % WIDTH == 0 {
                raw_index += 1;
            }

            let index = j * WIDTH;

            let perm_slice = &row[index..index + WIDTH];
            let result = scalar::perm(perm_slice.to_vec())?[1];

            row[raw_index] = result;
        }

        // Set the skipped bitflags
        for j in 0..merkle {
            row[j * WIDTH] = full_flags;
        }

        merkle /= MERKLE_ARITY;
    }

    Ok(row[1])
}

/// Normalize the input.
///
/// The result is a slice with the width of the merkle tree and the bitflags.
///
/// The absent leaves will be zeroed.
fn input_to_merkle_row(data: &[Option<Scalar>]) -> [Scalar; MERKLE_INNER_WIDTH] {
    let mut leaves = [Scalar::zero(); MERKLE_INNER_WIDTH];

    for i in 0..MERKLE_WIDTH / MERKLE_ARITY {
        let mut bitflag = 0u64;

        for j in 0..MERKLE_ARITY {
            let data_offset = i * MERKLE_ARITY + j;
            if data_offset >= data.len() {
                break;
            }

            if let Some(l) = data[data_offset] {
                leaves[i * WIDTH + j + 1] = l;
                bitflag |= 1u64 << MERKLE_ARITY - j - 1;
            }
        }

        leaves[i * WIDTH] = Scalar::from(bitflag);
    }

    leaves
}

#[cfg(test)]
mod tests {
    use crate::scalar::{merkle, Scalar};
    use crate::*;

    #[test]
    fn merkle_row() {
        // Build a vec with a full merkle row
        let mut expected: Vec<Scalar> = std::iter::repeat(())
            .take(MERKLE_INNER_WIDTH)
            .enumerate()
            .map(|(i, _)| Scalar::from(i as u64))
            .collect();

        // Set the bitflag in the leading leaves
        for i in 0..MERKLE_WIDTH / MERKLE_ARITY {
            expected[WIDTH * i] = Scalar::from((1u64 << MERKLE_ARITY) - 1);
        }

        // Collect elements, except leading bitflags
        let mut v: Vec<Option<Scalar>> = expected
            .iter()
            .enumerate()
            .filter_map(|(i, l)| if i % WIDTH == 0 { None } else { Some(Some(*l)) })
            .collect();

        // Remove the second element
        v[1] = None;
        expected[2] = Scalar::zero();
        expected[0] = Scalar::from(((1u64 << MERKLE_ARITY) - 1) ^ 1u64 << MERKLE_ARITY - 2);

        // Remove the entire second block
        for i in 0..MERKLE_ARITY + 1 {
            expected[MERKLE_ARITY + i + 1] = Scalar::zero();
        }
        for i in 0..MERKLE_ARITY {
            v[MERKLE_ARITY + i] = None;
        }

        // Remove the last element
        v[MERKLE_WIDTH - 1] = None;
        expected[MERKLE_INNER_WIDTH - 1] = Scalar::zero();

        // Remove the pre-last element
        v[MERKLE_WIDTH - 2] = None;
        expected[MERKLE_INNER_WIDTH - 2] = Scalar::zero();

        // Adjust the bitflag
        expected[MERKLE_INNER_WIDTH - WIDTH] =
            Scalar::from((((1u64 << MERKLE_ARITY) - 1) << 2) & ((1u64 << MERKLE_ARITY) - 1));

        let row = merkle::input_to_merkle_row(v.as_slice());

        assert_eq!(&expected[..], &row[..]);
    }

    #[test]
    fn merkle_one() {
        // Build a vec with a full merkle row
        let mut expected = vec![Scalar::zero(); MERKLE_INNER_WIDTH];
        expected[0] = Scalar::from(1u64 << (MERKLE_ARITY - 1));
        expected[1] = Scalar::one();

        // Set the first element
        let mut v: Vec<Option<Scalar>> = vec![None; MERKLE_INNER_WIDTH];
        v[0] = Some(Scalar::one());

        let row = merkle::input_to_merkle_row(v.as_slice());

        assert_eq!(&expected[..], &row[..]);
    }

    #[test]
    fn merkle_partial_row() {
        // Build a vec with a full merkle row
        let mut expected: Vec<Scalar> = std::iter::repeat(())
            .take(MERKLE_INNER_WIDTH)
            .enumerate()
            .map(|(i, _)| Scalar::from(i as u64))
            .collect();

        // Set the bitflag in the leading leaves
        for i in 0..MERKLE_WIDTH / MERKLE_ARITY {
            expected[WIDTH * i] = Scalar::from((1u64 << MERKLE_ARITY) - 1);
        }

        // Collect elements, except leading bitflags
        let mut v: Vec<Option<Scalar>> = expected
            .iter()
            .enumerate()
            .filter_map(|(i, l)| if i % WIDTH == 0 { None } else { Some(Some(*l)) })
            .collect();

        // Remove the last block
        for i in 0..WIDTH {
            expected[MERKLE_INNER_WIDTH - i - 1] = Scalar::zero();
        }
        for i in 0..MERKLE_ARITY {
            v[MERKLE_WIDTH - i - 1] = None;
        }

        let row = merkle::input_to_merkle_row(v.as_slice());

        assert_eq!(&expected[..], &row[..]);
    }

    #[test]
    fn merkle_full_row() {
        // Build a vec with a full merkle row
        let mut expected: Vec<Scalar> = std::iter::repeat(())
            .take(MERKLE_INNER_WIDTH)
            .enumerate()
            .map(|(i, _)| Scalar::from(i as u64))
            .collect();

        // Set the bitflag in the leading leaves
        for i in 0..MERKLE_WIDTH / MERKLE_ARITY {
            expected[WIDTH * i] = Scalar::from((1u64 << MERKLE_ARITY) - 1);
        }

        // Collect elements, except leading bitflags
        let v: Vec<Option<Scalar>> = expected
            .iter()
            .enumerate()
            .filter_map(|(i, l)| if i % WIDTH == 0 { None } else { Some(Some(*l)) })
            .collect();

        let row = merkle::input_to_merkle_row(v.as_slice());

        assert_eq!(&expected[..], &row[..]);
    }

    #[test]
    fn merkle() {
        let result = merkle::root(&[Some(Scalar::one())]).unwrap();
        assert_ne!(Scalar::zero(), result)
    }

    #[test]
    fn merkle_pad() {
        let result = merkle::root(&[Some(Scalar::one())]).unwrap();
        assert_ne!(
            result,
            merkle::root(&[Some(Scalar::one()), Some(Scalar::zero())]).unwrap()
        );
    }

    #[test]
    fn merkle_det() {
        let mut rng = rand::thread_rng();
        let v: Vec<Option<Scalar>> = std::iter::repeat(Some(Scalar::random(&mut rng)))
            .take(MERKLE_ARITY)
            .collect();

        let result = merkle::root(v.as_slice()).unwrap();
        assert_eq!(result, merkle::root(v.as_slice()).unwrap());

        let v: Vec<Option<Scalar>> = std::iter::repeat(Some(Scalar::random(&mut rng)))
            .take(MERKLE_WIDTH)
            .collect();

        assert_ne!(result, merkle::root(v.as_slice()).unwrap());
    }

    #[test]
    fn merkle_sanity_proof() {
        let base = Scalar::one();
        let root = merkle::root(&[Some(base)]).unwrap();

        let mut main_path = merkle::hash(&[Some(base)]).unwrap();
        let mut round_void = merkle::hash(&[]).unwrap();
        let mut void: Vec<Option<Scalar>> = std::iter::repeat(Some(round_void))
            .take(MERKLE_ARITY)
            .collect();

        for _ in 0.._MERKLE_HEIGHT - 2 {
            round_void = merkle::hash(void.as_slice()).unwrap();
            void[0] = Some(main_path);
            main_path = merkle::hash(void.as_slice()).unwrap();
            void = std::iter::repeat(Some(round_void))
                .take(MERKLE_ARITY)
                .collect();
        }

        assert_eq!(root, main_path);
    }
}
