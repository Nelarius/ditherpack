#![deny(
    clippy::pedantic,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]

use anyhow::{Context, Result};
use clap::Parser;
use minifb::{Key, Window, WindowOptions};
use std::path::PathBuf;

#[derive(Parser)]
struct CliOpts {
    #[clap(short, parse(from_os_str))]
    input_image: PathBuf,
}

fn main() -> Result<()> {
    let opts = CliOpts::parse();
    let file = std::fs::File::open(&opts.input_image)
        .with_context(|| format!("Opening file: {}", opts.input_image.display()))?;

    let rgb_img = ditherpack::unpack(file)
        .with_context(|| format!("Unpacking image: {}", opts.input_image.display()))?;

    let width = rgb_img.dimensions.0 as usize;
    let height = rgb_img.dimensions.1 as usize;
    let mut window = Window::new(
        "Ditherpack inspector",
        width,
        height,
        WindowOptions::default(),
    )
    .with_context(|| "Opening window")?;

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&rgb_img.pixels, width, height)?;
    }

    Ok(())
}
