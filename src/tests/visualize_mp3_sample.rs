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

use crate::deinterleave_stereo;
use crate::tests::testutil::{TEST_OUT_DIR, TEST_SAMPLES_DIR, decode_mp3};
use crate::waveform::Waveform;
use std::path::Path;

#[test]
fn visualize_mp3_sample() {
    let lrlr_samples = decode_mp3(&Path::new(TEST_SAMPLES_DIR).join("sample_1.mp3"));
    let (left, right) = deinterleave_stereo(&lrlr_samples);

    for (samples, name) in [(left, "left"), (right, "right")] {
        Waveform::new(&samples)
            .sample_rate(44100.0)
            .title(format!("sample_1.mp3 ({name} channel)"))
            .write_png(format!("{TEST_OUT_DIR}/sample_1_waveform_{name}.png"))
            .unwrap();
    }
}
