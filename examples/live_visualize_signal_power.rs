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
use audio_visualizer::live::{LiveVisualizer, Transform};

mod common;

/// Live visualization of the signal power of real-time audio.
/// **Execute this with `--release`, otherwise it is very laggy!**
fn main() {
    // Signal power of the latest ~12ms (at 44.1kHz), computed over the whole
    // audio history so the curve scrolls from right (now) to left (past).
    let to_power = |samples: &[f32], sample_rate: f32| {
        const WINDOW: usize = 512;
        let (chunks, _) = samples.as_chunks::<WINDOW>();
        chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| {
                let power = chunk.iter().map(|s| (*s as f64).powi(2)).sum::<f64>() / WINDOW as f64;
                let time = ((i + 1) * WINDOW) as f64 / sample_rate as f64
                    - samples.len() as f64 / sample_rate as f64;
                (time, power)
            })
            .collect::<Vec<_>>()
    };

    let input = common::select_input();
    LiveVisualizer::new(Transform::points(to_power))
        .title("Live Signal Power View")
        .axis_labels("time (seconds)", "signal power")
        .y_range(0.0..0.25)
        .input(input)
        .open()
        .unwrap();
}
