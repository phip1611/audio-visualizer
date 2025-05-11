use std::fs::File;
use std::path::Path;
use symphonia::core::audio::{AudioBuffer, Signal};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
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
        .format(
            &Hint::default(),
            mss,
            &Default::default(),
            &Default::default(),
        )
        .unwrap();
    let mut format_reader = probed.format;
    let track = format_reader.tracks().first().unwrap();
    let mut decoder = get_codecs()
        .make(&track.codec_params, &Default::default())
        .unwrap();

    let mut audio_data_lrlr = Vec::new();
    while let Ok(packet) = format_reader.next_packet() {
        if let Ok(audio_buf_ref) = decoder.decode(&packet) {
            let audio_spec = audio_buf_ref.spec();
            let mut audio_buf_i16 =
                AudioBuffer::<i16>::new(audio_buf_ref.frames() as u64, *audio_spec);
            audio_buf_ref.convert(&mut audio_buf_i16);

            match audio_spec.channels.count() {
                2 => {
                    let iter = audio_buf_i16
                        .chan(0)
                        .iter()
                        .zip(audio_buf_i16.chan(1))
                        // LRLR interleavment
                        .flat_map(|(&l, &r)| [l, r]);
                    //.map(|(&l, &r)| ((l as i32 + r as i32) / 2) as i16);
                    audio_data_lrlr.extend(iter);
                }
                n => panic!("Unsupported amount of channels: {n}"),
            }
        }
    }
    audio_data_lrlr
}
