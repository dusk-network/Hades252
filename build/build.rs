mod ark;
mod mds;

pub use bls12_381::Scalar;

struct RawScalar(pub [u64; 4]);
impl From<Scalar> for RawScalar {
    fn from(s: Scalar) -> Self {
        unsafe { std::mem::transmute(s) }
    }
}

fn main() -> std::io::Result<()> {
    ark::write_to("assets/ark.bin")?;
    mds::write_to("assets/mds.bin")?;
    Ok(())
}
