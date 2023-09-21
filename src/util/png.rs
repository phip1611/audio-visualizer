/*
MIT License

Copyright (c) 2021 Philipp Schuster

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Writes RGB-bytes into the given file using [`png`]-crate.
pub fn write_png_file_u8(file: &Path, rgb_data: &[u8], image_width: u32, image_height: u32) {
    let file = File::create(file).unwrap();
    let mut writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(&mut writer, image_width, image_height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(rgb_data).unwrap();
}

/// Wrapper around [`write_png_file_u8`] that takes a vector of vectors with RGB-tuples.
/// (rows, cols).
pub fn write_png_file_rgb_tuples(file: &Path, rgb_image: &[Vec<(u8, u8, u8)>]) {
    let width = rgb_image[0].len() as u32;
    let height = rgb_image.len() as u32;

    // data must be RGBA sequence: RGBARGBARGBA...
    let rgb_data = rgb_image
        .iter()
        // get iter over each row
        .flat_map(|row| row.iter())
        .flat_map(|(r, g, b)| vec![r, g, b].into_iter())
        .copied()
        .collect::<Vec<u8>>();

    write_png_file_u8(file, &rgb_data, width, height)
}
