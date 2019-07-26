use crate::errors::PermError;
use crate::permutation::Permutation;
use curve25519_dalek::scalar::Scalar;

// hash is a thin layer over the permutation struct
pub struct Hash {
    perm: Permutation,
}

impl Hash {
    fn new() -> Self {
        let mut p = Permutation::default();

        // First value is zero
        p.data.push(Scalar::from(0 as u8));

        Hash { perm: p }
    }

    pub fn input(&mut self, s: Scalar) -> Result<(), PermError> {
        self.perm.input(s)
    }

    pub fn reset(&mut self) {
        self.perm.reset();
        self.perm.data.push(Scalar::from(0 as u8))
    }

    pub fn input_bytes(&mut self, bytes: &[u8]) -> Result<(), PermError> {
        self.perm.input_bytes(bytes)
    }
    pub fn inputs(&mut self, scalars: Vec<Scalar>) -> Result<(), PermError> {
        self.perm.inputs(scalars)
    }

    fn pad(&mut self) {
        let pad_amount = self.perm.width_left();
        let zero = Scalar::from(0 as u8);
        let zeroes = vec![zero; pad_amount];

        self.perm.data.extend(zeroes)
    }

    pub fn result(&mut self) -> Option<[u8; 32]> {
        // Pad remaining width with zero
        self.pad();

        // Apply permutation
        let words = self.perm.result().ok();
        match words {
            Some(words) => Some(words[1].to_bytes()),
            None => None,
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use hex::*;

    #[test]
    fn test_hash_reset() {
        let mut h = Hash::new();
        h.input_bytes(b"hello").unwrap();
        h.input_bytes(b"world").unwrap();
        let digest = h.result().unwrap();
        assert_eq!(
            "0d36d39f735454d48fa44e68297b2ade3ad3ad06246ebdde0aa1fe554666780f",
            hex::encode(digest)
        );

        h.reset();

        h.input_bytes(b"hello").unwrap();
        h.input_bytes(b"world").unwrap();
        let digest = h.result().unwrap();
        assert_eq!(
            "0d36d39f735454d48fa44e68297b2ade3ad3ad06246ebdde0aa1fe554666780f",
            hex::encode(digest)
        );
    }

}
