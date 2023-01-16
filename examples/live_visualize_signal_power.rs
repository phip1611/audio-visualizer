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
use ringbuffer::{AllocRingBuffer, RingBufferExt, RingBufferWrite};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencyValue};
use std::cell::{Cell, RefCell};
use std::cmp::max;
use std::io::{stdin, BufRead};
use std::time::Instant;

/// Example that creates a live visualization of the audio signal power of realtime audio data
/// **Execute this with `--release`, otherwise it is very laggy!**.
fn main() {
    let epoch = Cell::new(Instant::now());

    let power_history: RefCell<AllocRingBuffer<(f64, f64)>> =
        RefCell::new(AllocRingBuffer::with_capacity(2_usize.pow(12)));

    // Closure that captures `visualize_spectrum`.
    let to_power_fn = move |audio: &[f32], sampling_rate: f32| {
        let elapsed_s = epoch.get().elapsed().as_secs_f64();

        // equals 11.6ms with 44.1kHz sampling rate or 10.7ms with 48kHz sampling rate.
        const NUM_SAMPLES: usize = 256;
        let skip_elements = audio.len() - NUM_SAMPLES;
        // spectrum analysis only of the latest 46ms
        let relevant_samples = &audio[skip_elements..];

        let power_sum = relevant_samples
            .iter()
            .copied()
            .map(|x| x * x)
            .fold(0.0, |acc, val| acc + val as f64);
        let power = power_sum / relevant_samples.len() as f64;


        let mut power_history = power_history.borrow_mut();
        let mut power_history_vec = power_history.to_vec();
        power_history_vec.iter_mut().for_each(|(time, _val)| {
            *time -= elapsed_s;
        });
        power_history_vec.push((0.0 - 0.01, power));
        power_history.extend(power_history_vec.clone());
        /*loop {
            match power_history.iter_mut().next() {
                None => break,
                Some((time, _val)) => {
                    dbg!("WAH");
                    *time -= elapsed_s;
                }
            }
        }
        dbg!("WAH");
        power_history.push((0.0, power));*/


        epoch.replace(Instant::now());
        power_history_vec
    };

    let in_dev = select_input_dev();
    open_window_connect_audio(
        "Live Spectrum View",
        None,
        None,
        // 0.0..22050.0_f64.log(100.0),
        Some(-5.9..0.0),
        Some(0.0..0.25),
        "x-axis",
        "y-axis",
        AudioDevAndCfg::new(Some(in_dev), None),
        TransformFn::Complex(&to_power_fn),
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
