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

use crate::Channels;
use plotters::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Visualizes audio as a waveform in a png file using "plotters" crate.
/// If the data is stereo, it creates two files (with "left_" and "right_" prefix).
pub fn waveform_static_plotters_png_visualize(
    samples: &[i16],
    channels: Channels,
    directory: &str,
    filename: &str,
) {
    if channels.is_stereo() {
        assert_eq!(
            0,
            samples.len() % 2,
            "If stereo is provided, the length of the audio data must be even!"
        );
        let (left, right) = channels.stereo_interleavement().to_channel_data(samples);
        waveform_static_plotters_png_visualize(
            &left,
            Channels::Mono,
            directory,
            &format!("left_{}", filename),
        );
        waveform_static_plotters_png_visualize(
            &right,
            Channels::Mono,
            directory,
            &format!("right_{}", filename),
        );
        return;
    }

    if !fs::exists(directory).unwrap() {
        fs::create_dir(directory).unwrap();
    }
    let mut path = PathBuf::new();
    path.push(directory);
    path.push(filename);

    let mut max = 0;
    for sample in samples {
        let sample = *sample as i32;
        let sample = sample.abs();
        if sample > max {
            max = sample;
        }
    }

    let width = (samples.len() / 5) as u32;
    let width = if width > 4000 { 4000 } else { width };
    let root = BitMapBackend::new(&path, (width, 1000)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption("y=music(t)", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0..samples.len() as f32, -max as f32..max as f32)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            // (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
            samples
                .iter()
                .enumerate()
                .map(|(sample_i, amplitude)| (sample_i as f32, *amplitude as f32)),
            &RED,
        ))
        .unwrap()
        // .label("y = music(t)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::testutil::{decode_mp3, TEST_OUT_DIR, TEST_SAMPLES_DIR};
    use crate::ChannelInterleavement;

    #[test]
    fn test_visualize_png_output() {
        let mut path = PathBuf::new();
        path.push(TEST_SAMPLES_DIR);
        path.push("sample_1.mp3");

        let lrlr_mp3_samples = decode_mp3(path.as_path());

        waveform_static_plotters_png_visualize(
            &lrlr_mp3_samples,
            Channels::Stereo(ChannelInterleavement::LRLR),
            TEST_OUT_DIR,
            "waveform_static_plotters_png_visualize_example.png",
        );
    }
}
