mod ark;
mod mds;

pub use algebra::fields::jubjub::fq::Fq;

fn main() -> std::io::Result<()> {
    ark::write_to("assets/ark.bin")?;
    mds::write_to("assets/mds.bin")?;
    Ok(())
}
