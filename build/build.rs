mod ark;
mod mds;

pub use bls12_381::Scalar as BlsScalar;

fn main() -> std::io::Result<()> {
    ark::write_to("assets/ark.bin")?;
    mds::write_to("assets/mds.bin")?;
    Ok(())
}
