use std::cell::RefCell;
use std::cmp::max;
use audio_visualizer::dynamic::live_input_visualize::visualize_minifb::setup_window;
use audio_visualizer::dynamic::live_input_visualize::{
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
