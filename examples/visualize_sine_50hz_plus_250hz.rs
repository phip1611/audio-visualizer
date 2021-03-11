use audio_visualizer::Channels;
use audio_visualizer::waveform::staticc::png_file::waveform_static_png_visualize;
use audio_visualizer::test_support::TEST_OUT_DIR;
use audio_visualizer::util::sine::sine_wave_audio_data_multiple;

fn main() {
    let sampling_rate = 44100;
    let duration_ms = 100;
    let sin_audio_sum = sine_wave_audio_data_multiple(
        // 50Hz in 100ms => sin wave will have five time periods
        // 250Hz in 100ms => sin wave will have twenty-five time periods
        &vec![50_f64, 250_f64],
        sampling_rate,
        duration_ms
    );
    waveform_static_png_visualize(
        &sin_audio_sum,
        Channels::Mono,
        TEST_OUT_DIR,
        "sinus-wave-50hz_plus_250hz.png"
    )
}

