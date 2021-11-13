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
//! Static waveform visualization which exports the waveform to a PNG file.

use crate::util::png::write_png_file_rgb_tuples;
use crate::Channels;
use std::path::PathBuf;

/// Visualizes audio as a waveform in a png file in the most simple way.
/// There are no axes. If the audio data is mono, it creates one file.
/// If the data is stereo, it creates two files (with "left_" and "right_" prefix).
pub fn waveform_static_png_visualize(
    samples: &[i16],
    channels: Channels,
    directory: &str,
    filename: &str,
) {
    let image_width = 1500;
    let image_height = 200;
    if channels.is_stereo() {
        assert_eq!(
            0,
            samples.len() % 2,
            "If stereo is provided, the length of the audio data must be even!"
        );
        let (left, right) = channels.stereo_interleavement().to_channel_data(samples);
        waveform_static_png_visualize(
            &left,
            Channels::Mono,
            directory,
            &format!("left_{}", filename),
        );
        waveform_static_png_visualize(
            &right,
            Channels::Mono,
            directory,
            &format!("right_{}", filename),
        );
        return;
    }

    // needed for offset calculation; width per sample
    let width_per_sample = image_width as f64 / samples.len() as f64;
    // height in pixel per possible value of a sample; counts in that the y axis lays in the middle
    let height_per_max_amplitude = image_height as f64 / 2_f64 / i16::max_value() as f64;

    // RGB image data
    let mut image = vec![vec![(255, 255, 255); image_width]; image_height];
    for (sample_index, sample_value) in samples.iter().enumerate() {
        // x offset; from left
        let x = (sample_index as f64 * width_per_sample) as usize;
        // y offset; from top
        // image_height/2: there is our y-axis
        let sample_value = *sample_value as f64 * -1.0; // y axis grows downwards
        let mut y = ((image_height / 2) as f64 + sample_value * height_per_max_amplitude) as usize;

        // due to rounding it can happen that we get out of bounds
        if y == image_height {
            y -= 1;
        }

        image[y][x] = (0, 0, 0);
    }

    let mut path = PathBuf::new();
    path.push(directory);
    path.push(filename);
    write_png_file_rgb_tuples(&path, &image);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::testutil::{TEST_OUT_DIR, TEST_SAMPLES_DIR};
    use crate::ChannelInterleavement;
    use minimp3::{Decoder as Mp3Decoder, Error as Mp3Error, Frame as Mp3Frame};
    use std::fs::File;

    /// This test works, if it doesn't panic.
    #[test]
    fn test_no_out_of_bounds_panic() {
        let audio_data = vec![i16::MAX, i16::MIN];
        waveform_static_png_visualize(
            &audio_data,
            Channels::Mono,
            TEST_OUT_DIR,
            "sample_1_waveform-test-out-of-bounds-check.png",
        );
    }

    #[test]
    fn test_visualize_png_output() {
        let mut path = PathBuf::new();
        path.push(TEST_SAMPLES_DIR);
        path.push("sample_1.mp3");
        let mut decoder = Mp3Decoder::new(File::open(path).unwrap());

        let mut lrlr_mp3_samples = vec![];
        loop {
            match decoder.next_frame() {
                Ok(Mp3Frame {
                    data: samples_of_frame,
                    ..
                }) => {
                    for sample in samples_of_frame {
                        lrlr_mp3_samples.push(sample);
                    }
                }
                Err(Mp3Error::Eof) => break,
                Err(e) => panic!("{:?}", e),
            }
        }

        waveform_static_png_visualize(
            &lrlr_mp3_samples,
            Channels::Stereo(ChannelInterleavement::LRLR),
            TEST_OUT_DIR,
            "waveform_static_png_visualize_example.png",
        );
    }
}
