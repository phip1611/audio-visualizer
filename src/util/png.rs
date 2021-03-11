use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub fn write_png_file_u8(file: &Path, rgb_data: &[u8], image_width: u32, image_height: u32) {
    let file = File::create(file).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, image_width as u32, image_height as u32);
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&rgb_data).unwrap();
}

pub fn write_png_file_rgb_tuples(file: &Path, rgb_image: &Vec<Vec<(u8, u8, u8)>>) {
    let width = rgb_image[0].len() as u32;
    let height = rgb_image.len() as u32;

    // data must be RGBA sequence: RGBARGBARGBA...
    let rgb_data = rgb_image
        .into_iter()
        // get iter over each row
        .flat_map(|row| row.iter())
        .flat_map(|(r, g, b)| vec![r, g, b].into_iter())
        .map(|v| *v)
        .collect::<Vec<u8>>();

    write_png_file_u8(file, &rgb_data, width, height)
}
