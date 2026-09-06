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
use std::f64::consts::PI;

/// Creates a sine (sinus) wave function for a given frequency.
/// Don't forget to scale up the value to the audio resolution.
/// So far, amplitude is in interval `[-1; 1]`. The parameter
/// of the returned function is the point in time in seconds.
///
/// * `frequency` is in Hertz
pub fn sine_wave(frequency: f64) -> Box<dyn Fn(f64) -> f64> {
    Box::new(move |t| (t * frequency * 2.0 * PI).sin())
}

/// See [`sine_wave_audio_data_multiple`]
pub fn sine_wave_audio_data(frequency: f64, sampling_rate: u32, duration_ms: u32) -> Vec<f32> {
    sine_wave_audio_data_multiple(&[frequency], sampling_rate, duration_ms)
}

/// Like [`sine_wave_audio_data`] but puts multiple sinus waves on top of each
/// other. Returns the sum of the sine waves as amplitudes in `[-1.0, 1.0]`.
///
/// * `frequencies` frequencies in Hz for the sinus waves
/// * `sampling_rate` sampling rate, i.e. 44100Hz
/// * `duration_ms` duration of the audio data in milliseconds
pub fn sine_wave_audio_data_multiple(
    frequencies: &[f64],
    sampling_rate: u32,
    duration_ms: u32,
) -> Vec<f32> {
    let sine_waves = frequencies
        .iter()
        .map(|f| sine_wave(*f))
        .collect::<Vec<_>>();

    let sample_count = (sampling_rate as f64 * (duration_ms as f64 / 1000_f64)) as usize;
    (0..sample_count)
        .map(|i_sample| {
            let t = i_sample as f64 / sampling_rate as f64;
            let sum: f64 = sine_waves.iter().map(|wave| wave(t)).sum();
            // scale down to prevent harsh clipping when waves add up
            (sum * 0.6).clamp(-1.0, 1.0) as f32
        })
        .collect()
}
