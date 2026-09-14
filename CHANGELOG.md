# Changelog

## Unreleased (v0.7.0)

Complete overhaul of the crate: same functionality, new API, new rendering
stacks.

- **BREAKING** all functionality works on mono `f32` samples (amplitudes in
  `[-1.0, 1.0]`); the `Channels`/`ChannelInterleavement` enums are replaced
  by `deinterleave_stereo()` (LLRR interleavement support was dropped)
- **BREAKING** static visualizations: the `waveform::Waveform` and
  `spectrum::Spectrum` builders (PNG file + SVG string output) replace the
  four `*_static_*_visualize` functions; rendering uses the actively
  maintained [charts-rs](https://crates.io/crates/charts-rs) instead of the
  unmaintained plotters and the hand-rolled PNG plotting
- **BREAKING** live visualization: `live::LiveVisualizer` and
  `live::Transform` replace `dynamic::window_top_btm::open_window_connect_audio`
  and its nine positional parameters; the GUI uses
  [egui](https://crates.io/crates/egui)/eframe + egui_plot instead of
  minifb + plotters (axes and auto bounds included);
  `dynamic::live_input::AudioDevAndCfg` becomes `live::AudioInput`
- **BREAKING** fallible operations return `Result<_, audio_visualizer::Error>`
  instead of panicking
- **BREAKING** MSRV is 1.95.0
- transforms are `FnMut` closures now and may hold state across frames
- waveforms are rendered as a min/max envelope per pixel bucket, so peaks
  survive downsampling and large inputs render fast
- Rust edition is 2024
- updated all dependencies; removed plotters, plotters-bitmap, minifb, png
  and biquad (the lowpass filter example uses the latest lowpass-filter)

## v0.5.0 (2025-05-11)
- **BREAKING** MSRV is 1.81.0
- (slightly) modernized crate and dependencies
- updated dependencies

## v0.4.0 (2023-09-21)
- **BREAKING** MSRV is 1.63.0
- build fix
- dependency bumps

## v0.3.1 (2021-11-16)
- removed accidentally public export of internal module

## v0.3.0 (2021-11-13)
- MSRV is 1.56.1 stable (because of Rust edition 2021)
- breaking changes: changed module paths
- new functionality: live audio + GUI + customized view! see example: \
  **Real-time audio + lowpass filter (6.9MB GIF)** \
  ![Example visualization of real-time audio + lowpass filter](res/live_demo_lowpass_filter_green_day_holiday.gif "Example visualization of real-time audio + lowpass filter") \
On the top you see the original waveform of the song Holiday by Green Day. On the bottom you see the data after a
lowpass filter was applied. The beats are visible.
- internal code improvements
