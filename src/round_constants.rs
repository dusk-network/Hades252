#![allow(non_snake_case)]

use curve25519_dalek::scalar::Scalar;
use lazy_static::*;

lazy_static! {
  pub static ref ROUND_CONSTANTS: [Scalar; 960] = {
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
    let non_zero_element = ROUND_CONSTANTS.iter().find(|&&x| x == Scalar::zero());
    assert!(non_zero_element.is_none());

    let mut constants_iter = ROUND_CONSTANTS.iter();
    for _ in 0..960 {
      let x = constants_iter.next();

      assert!(x.is_some());
      assert_ne!(&Scalar::zero(), x.unwrap());
    }
  }
}
