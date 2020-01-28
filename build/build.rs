mod ark;
mod mds;
mod settings_width;

fn main() -> std::io::Result<()> {
    settings_width::write()?;
    mds::write_to("assets/mds.bin")?;
    ark::write_to("assets/ark.bin")?;
    Ok(())
}
