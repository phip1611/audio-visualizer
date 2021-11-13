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
use audio_visualizer::dynamic::live_input::list_input_devs;
use audio_visualizer::dynamic::window_top_btm::visualize_minifb::setup_window;
use audio_visualizer::dynamic::window_top_btm::{
    open_window_connect_audio, TransformFn, SAMPLING_RATE,
};
use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F32};
use cpal::traits::DeviceTrait;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencyValue};
use std::cell::RefCell;
use std::cmp::max;
use std::io::{stdin, BufRead, Read};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Example that creates a live visualization of realtime audio data
/// through a lowpass filter. **Execute this with `--release`, otherwise it is very laggy!**.
fn main() {
    let in_dev = select_input_dev();
    open_window_connect_audio(
        "Live Audio Biquad Lowpass Filter View",
        None,
        None,
        None,
        None,
        "time (seconds)",
        "Amplitude (with Biquad Lowpass filter)",
        Some(in_dev),
        // lowpass filter
        TransformFn::Basic(|vals| {
            // Cutoff and sampling frequencies
            let f0 = 80.hz();
            let fs = 44.1.khz();

            // Create coefficients for the biquads
            let coeffs =
                Coefficients::<f32>::from_params(Type::LowPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();
            let mut lowpassed_data = Vec::with_capacity(vals.len());
            let mut biquad_lpf = DirectForm1::<f32>::new(coeffs);
            vals.iter()
                .for_each(|val| lowpassed_data.push(biquad_lpf.run(*val)));
            lowpassed_data
        }),
    );
}

/// Helps to select an input device.
fn select_input_dev() -> cpal::Device {
    let mut devs = list_input_devs();
    assert!(!devs.is_empty(), "no input devices found!");
    println!();
    devs.iter().enumerate().for_each(|(i, (name, dev))| {
        println!(
            "  [{}] {} {:?}",
            i,
            name,
            dev.default_input_config().unwrap()
        );
    });
    let mut input = String::new();
    stdin().lock().read_line(&mut input).unwrap();
    let index = (&input[0..1]).parse::<usize>().unwrap();
    devs.remove(index).1
}
