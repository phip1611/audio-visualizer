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

use crate::spectrum::Spectrum;
use crate::tests::testutil::TEST_OUT_DIR;
use crate::tests::testutil::sine::sine_wave_audio_data_multiple;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};

#[test]
fn visualize_spectrum_sine_50hz_plus_250hz() {
    let sampling_rate = 44100;
    let audio = sine_wave_audio_data_multiple(&[50.0, 250.0], sampling_rate, 1000);

    let window = hann_window(&audio[0..4096]);
    let spectrum =
        samples_fft_to_spectrum(&window, sampling_rate, FrequencyLimit::Max(400.0), None).unwrap();
    let data = spectrum
        .data()
        .iter()
        .map(|(frequency, magnitude)| (frequency.val(), magnitude.val()))
        .collect::<Vec<_>>();

    Spectrum::new(&data)
        .highlight(50.0)
        .highlight(250.0)
        .title("Spectrum of 50 Hz + 250 Hz sine wave")
        .write_png(format!("{TEST_OUT_DIR}/spectrum_sine_50hz_plus_250hz.png"))
        .unwrap();
}
