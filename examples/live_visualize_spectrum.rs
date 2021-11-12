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

/// Example that creates a live visualization of the frequency spectrum of realtime audio data
/// **Execute this with `--release`, otherwise it is very laggy!**.
fn main() {
    // Contains the data for the spectrum to be visualized. It contains ordered pairs of
    // `(frequency, frequency_value)`. During each iteration, the frequency value gets
    // combined with `max(old_value * smoothing_factor, new_value)`.
    let mut visualize_spectrum: RefCell<Vec<(f64, f64)>> = RefCell::new(vec![(0.0, 0.0); 1024]);

    // Closure that captures `visualize_spectrum`.
    let to_spectrum_fn = move |audio: &[f32]| {
        let skip_elements = audio.len() - 2048;
        // spectrum analysis only of the latest 46ms
        let relevant_samples = &audio[skip_elements..skip_elements + 2048];

        // do FFT
        let hann_window = hann_window(relevant_samples);
        let latest_spectrum = samples_fft_to_spectrum(
            &hann_window,
            SAMPLING_RATE as u32,
            // everything below is totally irrelevant
            // (and especially in logarithmic view annoying, because the range takes so much space)
            FrequencyLimit::Min(20.0),
            Some(&divide_by_N),
        )
            .unwrap();

        // now smoothen the spectrum; old values are decreased a bit and replaced,
        // if the new value is higher
        latest_spectrum
            .data()
            .iter()
            .zip(visualize_spectrum.borrow_mut().iter_mut())
            .for_each(|((fr_new, fr_val_new), (fr_old, fr_val_old))| {
                // actually only required in very first iteration
                *fr_old = fr_new.val() as f64;
                let old_val = *fr_val_old * 0.84;
                let max = max(fr_val_new.clone() * (5000.0 as f32).into(), FrequencyValue::from(old_val as f32));
                *fr_val_old = max.val() as f64;
            });

        visualize_spectrum.borrow().clone()
    };

    open_window_connect_audio(
        "Test",
        None,
        None,
        // 0.0..22050.0_f64.log(100.0),
        0.0..22050.0,
        0.0..100.0,
        "x-axis",
        "y-axis",
        // lowpass filter
        /*TransformFn::Basic(|x| {
            let mut data = x.iter().map(|x| (*x * (i16::MAX) as f32) as i16).collect::<Vec<_>>();
            lowpass_filter::simple::sp::apply_lpf_i16_sp(&mut data, 44100, 120);
            data.iter().map(|x| *x as f32).map(|x| x / i16::MAX as f32).collect()
        })*/
        TransformFn::Complex(&to_spectrum_fn),
    );


}
