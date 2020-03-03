mod ark;
mod mds;

pub use algebra::{curves::bls12_381::Bls12_381 as Curve, fields::bls12_381::fr::Fr as Scalar};

fn main() -> std::io::Result<()> {
    ark::write_to("assets/ark.bin")?;
    mds::write_to("assets/mds.bin")?;
    Ok(())
}
