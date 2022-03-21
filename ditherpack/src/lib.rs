#![deny(
    clippy::pedantic,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]

use bitvec::prelude::*;
use image::GenericImageView;
use itertools::Itertools;
use rand::prelude::*;

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
        .expect("Failed to load embedded 128x128_blue.png")
        .grayscale();

        let dimensions = texture.dimensions();
        let matrix = (0..dimensions.0)
            .cartesian_product(0..dimensions.1)
            .map(|(x, y)| texture.get_pixel(x, y)[0])
            .collect();

        Self { dimensions, matrix }
    }

    fn white_noise((xdim, ydim): (u32, u32)) -> Self {
        let mut matrix = Vec::new();
        for _ in 0..(xdim * ydim) {
            matrix.push(rand::thread_rng().gen::<u8>());
        }

        Self {
            dimensions: (xdim, ydim),
            matrix,
        }
    }

    fn look_up(&self, x: u32, y: u32) -> u8 {
        let j = x % self.dimensions.0;
        let i = y % self.dimensions.1;
        let idx = (i * self.dimensions.0 + j) as usize;

        self.matrix[idx]
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DitheredImage {
    dimensions: (u32, u32),
    bits: BitVec<u64, Lsb0>,
}

fn dithered_rgb_image(
    threshold_matrix: ThresholdMatrix,
    img_luma: image::GrayImage,
) -> DitheredImage {
    let dimensions: (u32, u32) = img_luma.dimensions();
    let mut bits: BitVec<u64, Lsb0> =
        bitvec::vec::BitVec::with_capacity((dimensions.0 as usize) * (dimensions.1 as usize));

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

#[derive(clap::ArgEnum, Clone, Copy, Debug)]
pub enum DitherType {
    Bayer,
    BlueNoise,
    WhiteNoise,
}

pub struct RgbImage {
    pub dimensions: (u32, u32),
    pub pixels: Vec<u32>,
}

#[derive(thiserror::Error, Debug)]
pub enum DitherpackError {
    #[error(transparent)]
    Serialization(#[from] bincode::Error),
    #[error(transparent)]
    Compression(#[from] std::io::Error),
}

pub fn pack<W: std::io::Write>(
    image: &image::DynamicImage,
    method: DitherType,
    writer: &mut W,
) -> Result<(), DitherpackError> {
    let threshold_matrix = match method {
        DitherType::Bayer => ThresholdMatrix::bayer_matrix(3),
        DitherType::BlueNoise => ThresholdMatrix::blue_noise(),
        DitherType::WhiteNoise => ThresholdMatrix::white_noise(image.dimensions()),
    };

    let img_luma = image.to_luma8();
    let dithered_img = dithered_rgb_image(threshold_matrix, img_luma);

    let bytes = bincode::serialize(&dithered_img)?;
    // zstd supports compression levels 1 to 22.
    // Levels >= 20 require huge amounts of memory and should be used with caution.
    zstd::stream::copy_encode(std::io::Cursor::new(bytes), writer, 19)?;

    Ok(())
}

pub fn unpack<R: std::io::Read>(reader: R) -> Result<RgbImage, DitherpackError> {
    let dithered_img: DitheredImage = bincode::deserialize_from(reader)?;

    let mut pixels: Vec<u32> = Vec::new();
    for b in dithered_img.bits.into_iter() {
        let px = if b { 0xffffffff } else { 0xff000000 };
        pixels.push(px);
    }

    Ok(RgbImage {
        dimensions: dithered_img.dimensions,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use bitvec::prelude::*;

    #[test]
    fn into_vec_length() {
        let mut bits: BitVec<u32, Lsb0> = BitVec::with_capacity(128);
        bits.push(true);
        bits.push(false);

        let bytes = bits.into_vec();
        assert_eq!(1, bytes.len())
    }
}
