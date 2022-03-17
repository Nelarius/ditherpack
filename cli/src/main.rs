use anyhow::{Context, Result};
use clap::Parser;
use ditherpack::{pack, DitherType};
use std::path::PathBuf;

#[derive(Parser)]
struct CliOpts {
    #[clap(short, parse(from_os_str))]
    input_image: PathBuf,
    #[clap(short, arg_enum)]
    method: DitherType,
}

fn main() -> Result<()> {
    let opts = CliOpts::parse();
    let output_file = opts.input_image.with_extension("ditherpack");

    let user_img = image::open(&opts.input_image)
        .with_context(|| format!("Opening image: {}", opts.input_image.display()))?;
    let user_img = user_img.grayscale();

    let mut file = std::fs::File::create(&output_file)
        .with_context(|| format!("Creating output file: {}", output_file.display()))?;
    pack(&user_img, opts.method, &mut file).with_context(|| {
        format!(
            "Packing image: {}, with method: {:?}",
            opts.input_image.display(),
            opts.method
        )
    })?;

    Ok(())
}
