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
use audio_visualizer::waveform::png_file::waveform_static_png_visualize;
use audio_visualizer::ChannelInterleavement;
use audio_visualizer::Channels;
use minimp3::{Decoder as Mp3Decoder, Error as Mp3Error, Frame as Mp3Frame};
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let mut path = PathBuf::new();
    path.push("test/samples");
    path.push("sample_1.mp3");
    let mut decoder = Mp3Decoder::new(File::open(path).unwrap());

    let mut lrlr_mp3_samples = vec![];
    loop {
        match decoder.next_frame() {
            Ok(Mp3Frame {
                data: samples_of_frame,
                ..
            }) => {
                for sample in samples_of_frame {
                    lrlr_mp3_samples.push(sample);
                }
            }
            Err(Mp3Error::Eof) => break,
            Err(e) => panic!("{:?}", e),
        }
    }

    waveform_static_png_visualize(
        &lrlr_mp3_samples,
        Channels::Stereo(ChannelInterleavement::LRLR),
        "test/samples",
        "sample_1_waveform.png",
    );
}
