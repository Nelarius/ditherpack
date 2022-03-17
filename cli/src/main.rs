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

fn main() {
    // TODO: there is a lot of unwrap here

    let opts = CliOpts::parse();
    let output_file = opts.input_image.with_extension("ditherpack");

    let user_img = image::open(opts.input_image).unwrap();
    let user_img = user_img.grayscale();

    let mut file = std::fs::File::create(output_file).unwrap();
    pack(&user_img, opts.method, &mut file).unwrap();
}
