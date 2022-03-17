use ditherpack::{pack, DitherType};
use structopt::StructOpt;

use std::path::PathBuf;

#[derive(StructOpt)]
struct CliOpts {
    #[structopt(short, parse(from_os_str))]
    input_image: PathBuf,
    #[structopt(short)]
    method: DitherType,
}

fn main() {
    // TODO: there is a lot of unwrap here

    let opts = CliOpts::from_args();
    let output_file = opts.input_image.with_extension("ditherpack");

    let user_img = image::open(opts.input_image).unwrap();
    let user_img = user_img.grayscale();

    let mut file = std::fs::File::create(output_file).unwrap();
    pack(&user_img, opts.method, &mut file).unwrap();
}
