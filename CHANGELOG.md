# Changelog

# v0.5.0 (2025-05-11)
- **BREAKING** MSRV is 1.81.0
- (slightly) modernized crate and dependencies
- updated dependencies

# v0.4.0 (2023-09-21)
- **BREAKING** MSRV is 1.63.0
- build fix
- dependency bumps

# v0.3.1 (2021-11-16)
- removed accidentally public export of internal module

# v0.3.0 (2021-11-13)
- MSRV is 1.56.1 stable (because of Rust edition 2021)
- breaking changes: changed module paths
- new functionality: live audio + GUI + customized view! see example: \
  **Real-time audio + lowpass filter (6.9MB GIF)** \
  ![Example visualization of real-time audio + lowpass filter](res/live_demo_lowpass_filter_green_day_holiday.gif "Example visualization of real-time audio + lowpass filter") \
On the top you see the original waveform of the song Holiday by Green Day. On the bottom you see the data after a
lowpass filter was applied. The beats are visible.
- internal code improvements
