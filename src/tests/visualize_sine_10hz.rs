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

use crate::tests::testutil::sine::sine_wave_audio_data;
use crate::tests::testutil::TEST_OUT_DIR;
use crate::waveform::png_file::waveform_static_png_visualize;
use crate::Channels;

#[test]
fn visualize_sine_10hz() {
    let frequency = 10_f64;
    let sampling_rate = 44100;
    let duration_ms = 1000;
    // we expect 10 time periods of the sine wav in the time interval
    let audio_signal = sine_wave_audio_data(frequency, sampling_rate, duration_ms);
    waveform_static_png_visualize(
        &audio_signal,
        Channels::Mono,
        TEST_OUT_DIR,
        "sinus-wave-10hz.png",
    )
}
