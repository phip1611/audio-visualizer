use std::fs::File;
use std::path::Path;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::TrackType;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::default::{get_codecs, get_probe};

pub mod sine;

/// Directory with test samples (e.g. mp3) can be found here.
pub const TEST_SAMPLES_DIR: &str = "test/samples";
/// If tests create files, they should be stored here.
pub const TEST_OUT_DIR: &str = "target/test_out";

/// Returns an MP3 as decoded i16 samples and with LRLR interleavement.
pub fn decode_mp3(file: &Path) -> Vec<i16> {
    let file = File::open(file).unwrap();
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let probed = get_probe()
        .probe(
            &Hint::default(),
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .unwrap();
    let mut format_reader = probed;
    let track = format_reader.default_track(TrackType::Audio).unwrap();
    let mut decoder = get_codecs()
        .make_audio_decoder(
            track.codec_params.as_ref().unwrap().audio().unwrap(),
            &AudioDecoderOptions::default(),
        )
        .unwrap();

    let mut audio_data_lrlr = Vec::new();
    while let Ok(Some(packet)) = format_reader.next_packet() {
        if let Ok(audio_buf_ref) = decoder.decode(&packet) {
            let audio_spec = audio_buf_ref.spec();

            match audio_spec.channels().count() {
                2 => {
                    audio_buf_ref.copy_to_vec_interleaved(&mut audio_data_lrlr);
                }
                n => panic!("Unsupported amount of channels: {n}"),
            }
        }
    }
    audio_data_lrlr
}
