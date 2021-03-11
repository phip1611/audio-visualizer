use audio_visualizer::Channels;
use audio_visualizer::waveform::staticc::png_file::waveform_static_png_visualize;
use audio_visualizer::util::sine::sine_wave_audio_data;
use audio_visualizer::test_support::TEST_OUT_DIR;

fn main() {
    let frequency = 10_f64;
    let sampling_rate = 44100;
    let duration_ms = 1000;
    // we expect 10 time periods of the sine wav in the time interval
    let audio_signal = sine_wave_audio_data(frequency, sampling_rate, duration_ms);
    waveform_static_png_visualize(
        &audio_signal,
        Channels::Mono,
        TEST_OUT_DIR,
        "sinus-wave-10hz.png"
    )
}

