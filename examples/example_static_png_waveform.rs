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
use crate::test_util::decode_mp3;
use audio_visualizer::deinterleave_stereo;
use audio_visualizer::waveform::Waveform;
use std::path::Path;

#[allow(unused)]
#[path = "../src/tests/testutil/mod.rs"]
mod test_util;

fn main() {
    let lrlr_mp3_samples = decode_mp3(Path::new("test/samples/sample_1.mp3"));
    let (left, right) = deinterleave_stereo(&lrlr_mp3_samples);

    for (samples, name) in [(left, "left"), (right, "right")] {
        Waveform::new(&samples)
            .sample_rate(44100.0)
            .title(format!("sample_1.mp3 ({name} channel)"))
            .write_png(format!("target/test_out/sample_1_waveform_{name}.png"))
            .unwrap();
    }
}
