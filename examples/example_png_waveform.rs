use minimp3::{Decoder as Mp3Decoder, Frame as Mp3Frame, Error as Mp3Error};
use audio_visualizer::ChannelInterleavement;
use audio_visualizer::Channels;
use audio_visualizer::waveform::staticc::png_file::visualize;
use std::path::PathBuf;
use std::fs::File;

fn main() {
    let mut path = PathBuf::new();
    path.push("src/test/samples");
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

    visualize(
        &lrlr_mp3_samples,
        Channels::Stereo(ChannelInterleavement::LRLR),
        "src/test/out",
        "sample_1_waveform.png"
    );
}
