#![deny(
    clippy::pedantic,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]

use bytemuck::{bytes_of, cast_slice};
use image::GenericImageView;
use itertools::Itertools;

pub use bitvec;
pub use image;

struct ThresholdMatrix {
    dimensions: (u32, u32),
    matrix: Vec<u8>,
}

impl ThresholdMatrix {
    fn bayer_matrix(power_of_two: u32) -> Self {
        let n = 2_u32.pow(power_of_two);
        let num_elements = n * n;
        let norm_factor = 255_f32 / (num_elements as f32);
        let matrix = (0..n)
            .cartesian_product(0..n)
            .map(|(x, y)| {
                let xc = x ^ y;
                let yc = y;

                let mut v = 0;
                for p in (0..power_of_two).rev() {
                    // Interleaves the bits in reverse
                    let bit_idx = 2 * (power_of_two - p - 1);
                    v |= ((yc >> p) & 1) << bit_idx;
                    v |= ((xc >> p) & 1) << (bit_idx + 1);
                }
                (v as f32 * norm_factor) as u8
            })
            .collect();

        Self {
            dimensions: (n, n),
            matrix,
        }
    }

    fn blue_noise() -> Self {
        let texture = image::load(
            std::io::Cursor::new(include_bytes!("128x128_blue.png")),
            image::ImageFormat::Png,
        )
        .unwrap()
        .grayscale();

        let dimensions = texture.dimensions();
        let matrix = (0..dimensions.0)
            .cartesian_product(0..dimensions.1)
            .map(|(x, y)| texture.get_pixel(x, y)[0])
            .collect();

        Self { dimensions, matrix }
    }

    fn look_up(&self, x: u32, y: u32) -> u8 {
        let j = x % self.dimensions.0;
        let i = y % self.dimensions.1;
        let idx: usize = (i * self.dimensions.0 + j)
            .try_into()
            .expect("i * row-length + j does not fit into usize");

        self.matrix[idx]
    }
}

struct DitheredImage {
    dimensions: (u32, u32),
    bits: bitvec::vec::BitVec,
}

fn dithered_rgb_image(
    threshold_matrix: &ThresholdMatrix,
    img: &image::DynamicImage,
) -> DitheredImage {
    let dimensions: (u32, u32) = img.dimensions();
    let mut bits =
        bitvec::vec::BitVec::with_capacity((dimensions.0 as usize) * (dimensions.1 as usize));
    let img_luma = img.as_luma8().unwrap();

    for (x, y) in (0..dimensions.0).cartesian_product(0..dimensions.1) {
        let luma = img_luma.get_pixel(x, y)[0];
        if luma > threshold_matrix.look_up(x, y) {
            // white
            bits.push(true);
        } else {
            // black
            bits.push(false);
        }
    }

    DitheredImage { dimensions, bits }
}

pub enum DitherType {
    Bayer,
    BlueNoise,
}

impl std::str::FromStr for DitherType {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<DitherType, Self::Err> {
        match input {
            "bayer" => Ok(DitherType::Bayer),
            "blue" => Ok(DitherType::BlueNoise),
            _ => Err("Could not parse DitherType"),
        }
    }
}

pub fn pack<W: std::io::Write>(
    image: &image::DynamicImage,
    method: DitherType,
    writer: &mut W,
) -> std::io::Result<()> {
    let threshold_matrix = match method {
        DitherType::Bayer => ThresholdMatrix::bayer_matrix(3),
        DitherType::BlueNoise => ThresholdMatrix::blue_noise(),
    };
    let dithered_img = dithered_rgb_image(&threshold_matrix, image);

    writer.write(bytes_of(&dithered_img.dimensions.0))?;
    writer.write(bytes_of(&dithered_img.dimensions.1))?;
    // writer.write(cast_slice(dithered_img.bits[..]))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
