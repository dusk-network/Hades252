use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const DEFAULT_WIDTH: usize = 5;
const MIN_WIDTH: usize = 3;
const MAX_WIDTH: usize = 9;
static ENV_HADES252_WIDTH: &'static str = "HADES252_WIDTH";

pub(crate) fn get_width() -> usize {
    let valid_number = format!(
        "{} needs to be a positive integer between {} and {} (included)",
        ENV_HADES252_WIDTH, MIN_WIDTH, MAX_WIDTH
    );

    let width = env::var(ENV_HADES252_WIDTH)
        .map(|v| v.parse().expect(&valid_number))
        .unwrap_or(DEFAULT_WIDTH);

    assert!(width >= MIN_WIDTH, valid_number);
    assert!(width <= MAX_WIDTH, valid_number);
    width
}

pub(crate) fn write() -> std::io::Result<()> {
    println!("cargo:rerun-if-env-changed={}", ENV_HADES252_WIDTH);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("width.rs");
    let mut f = File::create(&dest_path)?;

    let width = get_width();

    let s = format!(
        r#"
            /// Maximum Distance Separable square matrix width.
            pub const WIDTH: usize = {};
        "#,
        width
    );

    f.write_all(s.as_bytes())?;
    Ok(())
}
