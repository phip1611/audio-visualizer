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
use audio_visualizer::dynamic::live_input::{list_input_devs, AudioDevAndCfg};
use audio_visualizer::dynamic::window_top_btm::{open_window_connect_audio, TransformFn};
use cpal::traits::DeviceTrait;
use std::io::{stdin, BufRead};

/// Example that creates a live visualization of realtime audio data
/// through a lowpass filter. **Execute this with `--release`, otherwise it is very laggy!**.
fn main() {
    let in_dev = select_input_dev();
    open_window_connect_audio(
        "Live Audio Lowpass Filter View",
        None,
        None,
        None,
        None,
        "time (seconds)",
        "Amplitude (with Lowpass filter)",
        AudioDevAndCfg::new(Some(in_dev), None),
        // lowpass filter
        TransformFn::Basic(|x, sampling_rate| {
            let mut data_f32 = x
                .iter()
                .map(|x| (*x * (i16::MAX) as f32) as i16)
                .collect::<Vec<_>>();
            lowpass_filter::simple::sp::apply_lpf_i16_sp(&mut data_f32, sampling_rate as u16, 80);
            data_f32
                .iter()
                .map(|x| *x as f32)
                .map(|x| x / i16::MAX as f32)
                .collect()
        }),
    );
}

/// Helps to select an input device.
fn select_input_dev() -> cpal::Device {
    let mut devs = list_input_devs();
    assert!(!devs.is_empty(), "no input devices found!");
    if devs.len() == 1 {
        return devs.remove(0).1;
    }
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
