[![Build Status](https://travis-ci.com/dusk-network/Hades252.svg?branch=master)](https://travis-ci.com/dusk-network/Hades252)
[![codecov](https://codecov.io/gh/dusk-network/Hades252/branch/master/graph/badge.svg)](https://codecov.io/gh/dusk-network/Hades252)
[![Repository](https://dusk-network.github.io/Hades252/repo-badge.svg)](https://github.com/dusk-network/Hades252)
[![Documentation](https://dusk-network.github.io/Hades252/badge.svg)](https://dusk-network.github.io/Hades252/index.html)

# Hades252

Implementation of Hades252 over the Ristretto Scalar field.

*Unstable* : No guarantees can be made regarding the API stability.

## Documentation

To generate the `Hades252` documentation:

```sh
make doc
make doc-internal
```

## Use

To import `Hades252`, add the following to the dependencies section of your project's `Cargo.toml`:

```toml
Hades252 = "0.2"
```

By default `Hades252` has a `width` equals to `5`.
It's possible to use an arbitrary value, between `3` and `9`, by setting the
environment variable `HADES252_WIDTH` to the desired number.

## Parameters

- p = `2^252 + 27742317777372353535851937790883648493`

- Security level is 126 bits

- width = 5

- Number of full rounds = 8 . There are four full rounds at the beginning and four full rounds at the end,
where each full round has `WIDTH` quintic S-Boxes.

- Number of partial rounds = 59, where each partial round has one quintic S-Box and (width-1) identity functions.

- Number of round constants = 960

## Example with permutation of scalars
```
use hades252::{Fq, ScalarStrategy, Strategy};

// Generate the inputs that will permute.
// The number of values we can input is equivalent to `WIDTH`

let input = vec![Fq::from(1u64); hades252::WIDTH];
let mut strategy = ScalarStrategy::new();

let mut output = input.clone();
strategy.perm(output.as_mut_slice());

assert_ne!(&input, &output);
assert_eq!(input.len(), output.len());

```

## Deviations

- Round constants for the full rounds are generated following: [https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py](https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py)
They are then mapped onto `Scalar`s in the Ristretto scalar field.

- The MDS matrix is a cauchy matrix, the method used to generate it, is noted in section "Concrete Instantiations Poseidon and Starkad"

## Reference

[https://eprint.iacr.org/2019/458.pdf](https://eprint.iacr.org/2019/458.pdf)
