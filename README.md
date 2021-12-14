![Build Status](https://github.com/dusk-network/hades252/workflows/Continuous%20integration/badge.svg)
[![Repository](https://img.shields.io/badge/github-hades252-blueviolet?logo=github)](https://github.com/dusk-network/hades252)
[![Documentation](https://img.shields.io/badge/docs-dusk--hades-blue?logo=rust)](https://docs.rs/dusk-hades/)

# Hades252

Implementation of Hades252 permutation algorithm over the Bls12-381 Scalar field.

## Documentation

To generate the `Hades252` documentation:

```sh
make doc
make doc-internal
```

## Use

To import `Hades252`, add the following to the dependencies section of your project's `Cargo.toml`:

```toml
dusk-hades = "0.16.0"
```

`Hades252` has a `width` equals to `5`; it's possible to use a different value,
see [How to generate the assets](assets/HOWTO.md).

## Parameters

- p = `0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001`

- Security level is 117 -120 bits of security [NCCG] bits.

- width = 5

- Number of full rounds = 8 . There are four full rounds at the beginning and four full rounds at the end,
  where each full round has `WIDTH` quintic S-Boxes.

- Number of partial rounds = 59, where each partial round has one quintic S-Box and (width-1) identity functions.

- Number of round constants = 960

## Example for `ScalarStrategy`

```rust
use dusk_bls12_381::BlsScalar;
use dusk_hades::{ScalarStrategy, Strategy, WIDTH};

// Generate the inputs that will permute.
// The number of values we can input is equivalent to `WIDTH`

let input = vec![BlsScalar::from(1u64); dusk_hades::WIDTH];
let mut output = input.clone();

let mut strategy = ScalarStrategy::new();
strategy.perm(output.as_mut_slice());

assert_ne!(&input, &output);
assert_eq!(input.len(), output.len());
```

## Example for `GadgetStrategy`

```rust,ignore
// Proving that we know the pre-image of a hades-252 hash.
use dusk_hades::{GadgetStrategy, Strategy, WIDTH};
use dusk_plonk::prelude::*;

// Setup OG params.
const CAPACITY: usize = 1 << 10;

let public_parameters = PublicParameters::setup(CAPACITY, &mut rand::thread_rng()).unwrap();
let (ck, vk) = public_parameters.trim(CAPACITY).unwrap();;

// Gen inputs
let mut inputs = [BlsScalar::one(); WIDTH];

let mut prover = Prover::new(b"Hades_Testing");

// Generate the witness data
let mut composer = prover.composer_mut();
let zero = TurboComposer::constant_zero();

let mut witness = [zero; WIDTH];
witness.iter_mut()
    .zip(inputs.iter())
    .for_each(|(w, i)| *w = composer.append_witness(*i));

// Perform the permutation in the circuit
GadgetStrategy::gadget(prover.composer_mut(), &mut witness);

// Now your composer has been filled with a hades permutation
// inside.
// Now you can build your proof or keep extending your circuit.
```

## Deviations

- Round constants for the full rounds are generated following: [https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py](https://extgit.iaik.tugraz.at/krypto/hadesmimc/blob/master/code/calc_round_numbers.py)

- The MDS matrix is a cauchy matrix, the method used to generate it, is noted in section "Concrete Instantiations Poseidon and Starkad"

## Reference

[https://eprint.iacr.org/2019/458.pdf](https://eprint.iacr.org/2019/458.pdf)
