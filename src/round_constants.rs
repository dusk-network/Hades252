#![allow(non_snake_case)]

use curve25519_dalek::scalar::Scalar;
use lazy_static::*;

lazy_static! {
    pub(crate) static ref ROUND_CONSTANTS: [Scalar; 960] = {
        let bytes = include_bytes!("ark.bin");

        unsafe { std::ptr::read(bytes.as_ptr() as *const _) }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_round_constants() {
        // Check each element is non-zero
        let zero = Scalar::zero();
        let has_zero = ROUND_CONSTANTS.iter().any(|&x| x == zero);
        assert!(!has_zero);
    }
}
