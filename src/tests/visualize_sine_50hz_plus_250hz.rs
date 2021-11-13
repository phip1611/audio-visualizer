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
use crate::tests::testutil::sine::sine_wave_audio_data_multiple;
use crate::tests::testutil::TEST_OUT_DIR;
use crate::waveform::png_file::waveform_static_png_visualize;
use crate::Channels;

#[test]
fn visualize_sine_50hz_plus_250hz() {
    let sampling_rate = 44100;
    let duration_ms = 100;
    let sin_audio_sum = sine_wave_audio_data_multiple(
        // 50Hz in 100ms => sin wave will have five time periods
        // 250Hz in 100ms => sin wave will have twenty-five time periods
        &vec![50_f64, 250_f64],
        sampling_rate,
        duration_ms,
    );
    waveform_static_png_visualize(
        &sin_audio_sum,
        Channels::Mono,
        TEST_OUT_DIR,
        "sinus-wave-50hz_plus_250hz.png",
    )
}
