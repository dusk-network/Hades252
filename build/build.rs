mod ark;
mod mds;

pub use dusk_plonk::prelude::BlsScalar;

fn main() -> std::io::Result<()> {
    ark::write_to("assets/ark.bin")?;
    mds::write_to("assets/mds.bin")?;
    Ok(())
}
