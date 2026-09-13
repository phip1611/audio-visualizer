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
use lowpass_filter::lowpass_filter_slice;

mod common;

/// Live visualization of real-time audio through a lowpass filter.
/// **Execute this with `--release`, otherwise it is very laggy!**
fn main() {
    let input = common::select_input();
    LiveVisualizer::new(Transform::waveform(|samples, sample_rate| {
        let mut samples = samples.to_vec();
        lowpass_filter_slice(&mut samples, sample_rate, 80.0);
        samples
    }))
    .title("Live Audio Lowpass Filter View")
    .axis_labels("time (seconds)", "amplitude (lowpass filtered)")
    .input(input)
    .open()
    .unwrap();
}
