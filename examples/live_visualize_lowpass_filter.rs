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
use std::cell::RefCell;
use std::cmp::max;
use audio_visualizer::dynamic::window_top_btm::visualize_minifb::setup_window;
use audio_visualizer::dynamic::window_top_btm::{
    open_window_connect_audio, TransformFn, SAMPLING_RATE,
};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencyValue};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Example that creates a live visualization of realtime audio data
/// through a lowpass filter. **Execute this with `--release`, otherwise it is very laggy!**.
fn main() {
    open_window_connect_audio(
        "Live Audio Lowpass Filter View",
        None,
        None,
        None,
        None,
        "time (seconds)",
        "Amplitude (with Lowpass filter)",
        None,
        // lowpass filter
        TransformFn::Basic(|x| {
            let mut data = x.iter().map(|x| (*x * (i16::MAX) as f32) as i16).collect::<Vec<_>>();
            lowpass_filter::simple::sp::apply_lpf_i16_sp(&mut data, 44100, 120);
            data.iter().map(|x| *x as f32).map(|x| x / i16::MAX as f32).collect()
        })
    );


}
