use minimp3::{Decoder as Mp3Decoder, Frame as Mp3Frame, Error as Mp3Error};
use audio_visualizer::ChannelInterleavement;
use audio_visualizer::Channels;
use audio_visualizer::waveform::staticc::png_file::waveform_static_png_visualize;
use std::path::PathBuf;
use std::fs::File;
use audio_visualizer::test_support::{TEST_SAMPLES_DIR, TEST_OUT_DIR};

fn main() {
    let mut path = PathBuf::new();
    path.push(TEST_SAMPLES_DIR);
    path.push("sample_1.mp3");
    let mut decoder = Mp3Decoder::new(File::open(path).unwrap());

    let mut lrlr_mp3_samples = vec![];
    loop {
        match decoder.next_frame() {
            Ok(Mp3Frame { data: samples_of_frame, .. }) => {
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
        TEST_OUT_DIR,
        "sample_1_waveform.png"
    );
}
