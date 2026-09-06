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
//! Live audio visualization: records audio from an input device and shows it
//! in a real-time GUI window ([`egui`](https://crates.io/crates/egui)).
//!
//! The upper half of the window shows the waveform of the last seconds of
//! audio. The lower half shows a custom [`Transform`] of that audio, e.g. a
//! lowpass-filtered waveform or a frequency spectrum. Start with
//! [`LiveVisualizer`]; see the `live_visualize_*` examples in `examples/`.
//!
//! **Run this only with `--release`, otherwise it is very laggy.**

mod input;

pub use input::AudioInput;

use crate::Error;
use crate::waveform::envelope;
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::ops::Range;
use std::sync::{Arc, Mutex};

/// How many seconds of the latest audio are kept and displayed.
const AUDIO_HISTORY_SECONDS: usize = 5;

/// Upper bound of waveform points per plot line; roughly one point per
/// horizontal pixel.
const MAX_POINTS: usize = 1600;

/// Closure type of [`Transform::Waveform`].
pub type WaveformFn<'a> = Box<dyn FnMut(&[f32], f32) -> Vec<f32> + 'a>;
/// Closure type of [`Transform::Points`].
pub type PointsFn<'a> = Box<dyn FnMut(&[f32], f32) -> Vec<(f64, f64)> + 'a>;

/// Transformation of the recorded audio, displayed in the lower half of the
/// [`LiveVisualizer`] window.
///
/// Called with the latest audio samples (mono amplitudes in `[-1.0, 1.0]`)
/// and the sample rate in Hz, once per rendered frame. The closures may hold
/// state (`FnMut`), e.g. for smoothing across frames.
#[expect(missing_debug_implementations)]
pub enum Transform<'a> {
    /// The output is a waveform on the same time axis as the input, e.g. a
    /// filtered version of it.
    Waveform(WaveformFn<'a>),
    /// The output is a series of arbitrary `(x, y)` points, e.g. a frequency
    /// spectrum.
    Points(PointsFn<'a>),
}

impl<'a> Transform<'a> {
    /// Creates a [`Transform::Waveform`].
    pub fn waveform(f: impl FnMut(&[f32], f32) -> Vec<f32> + 'a) -> Self {
        Self::Waveform(Box::new(f))
    }

    /// Creates a [`Transform::Points`].
    pub fn points(f: impl FnMut(&[f32], f32) -> Vec<(f64, f64)> + 'a) -> Self {
        Self::Points(Box::new(f))
    }
}

/// Builder that opens a GUI window showing the live waveform of an audio
/// input device along with a custom [`Transform`] of it.
///
/// # Example
/// ```no_run
/// use audio_visualizer::live::{LiveVisualizer, Transform};
///
/// LiveVisualizer::new(Transform::waveform(|samples, _sample_rate| {
///     samples.iter().map(|s| s * 0.5).collect()
/// }))
/// .title("Half amplitude")
/// .open()
/// .unwrap();
/// ```
#[allow(missing_debug_implementations)]
pub struct LiveVisualizer<'a> {
    transform: Transform<'a>,
    title: String,
    input: Option<AudioInput>,
    x_range: Option<Range<f64>>,
    y_range: Option<Range<f64>>,
    x_label: String,
    y_label: String,
    window_size: (f32, f32),
}

impl<'a> LiveVisualizer<'a> {
    /// Creates a live visualizer with the given transformation for the lower
    /// chart.
    #[must_use]
    pub fn new(transform: Transform<'a>) -> Self {
        Self {
            transform,
            title: "Live Audio Visualization".to_string(),
            input: None,
            x_range: None,
            y_range: None,
            x_label: String::new(),
            y_label: String::new(),
            window_size: (1280.0, 720.0),
        }
    }

    /// Sets the window title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the audio input to record from. Default: the system default
    /// input device.
    #[must_use]
    pub fn input(mut self, input: AudioInput) -> Self {
        self.input = Some(input);
        self
    }

    /// Fixes the x-axis range of the lower chart. Default: the same time
    /// axis as the waveform for [`Transform::Waveform`], automatic bounds
    /// for [`Transform::Points`].
    #[must_use]
    pub const fn x_range(mut self, range: Range<f64>) -> Self {
        self.x_range = Some(range);
        self
    }

    /// Fixes the y-axis range of the lower chart. See [`Self::x_range`].
    #[must_use]
    pub const fn y_range(mut self, range: Range<f64>) -> Self {
        self.y_range = Some(range);
        self
    }

    /// Sets the axis labels of the lower chart.
    #[must_use]
    pub fn axis_labels(mut self, x: impl Into<String>, y: impl Into<String>) -> Self {
        self.x_label = x.into();
        self.y_label = y.into();
        self
    }

    /// Sets the initial window size in logical pixels. Default: 1280x720.
    #[must_use]
    pub const fn window_size(mut self, width: f32, height: f32) -> Self {
        self.window_size = (width, height);
        self
    }

    /// Starts recording, opens the window and blocks until it is closed
    /// (close button or Escape key).
    pub fn open(self) -> Result<(), Error> {
        let input = match self.input {
            Some(input) => input,
            None => AudioInput::default_device()?,
        };
        let sample_rate = input.config().sample_rate as f32;

        // Must be a power of two (ringbuffer requirement); pre-filled so the
        // waveform always covers the whole time axis.
        let mut buf = AllocRingBuffer::new(
            (AUDIO_HISTORY_SECONDS * sample_rate as usize).next_power_of_two(),
        );
        buf.fill(0.0);
        let audio = Arc::new(Mutex::new(buf));

        let stream = input.build_stream(audio.clone())?;
        cpal::traits::StreamTrait::play(&stream)
            .map_err(|e| Error::Audio(format!("can't start recording: {e}")))?;

        let app = App {
            audio,
            sample_rate,
            transform: self.transform,
            x_range: self.x_range,
            y_range: self.y_range,
            x_label: self.x_label,
            y_label: self.y_label,
        };
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([self.window_size.0, self.window_size.1]),
            ..Default::default()
        };
        eframe::run_native(&self.title, options, Box::new(move |_cc| Ok(Box::new(app))))
            .map_err(|e| Error::Gui(e.to_string()))?;

        // dropped here, after the window is closed: recording stops
        drop(stream);
        Ok(())
    }
}

/// The [`eframe`] application: two vertically stacked plots, redrawn
/// continuously with the latest audio data.
struct App<'a> {
    audio: Arc<Mutex<AllocRingBuffer<f32>>>,
    sample_rate: f32,
    transform: Transform<'a>,
    x_range: Option<Range<f64>>,
    y_range: Option<Range<f64>>,
    x_label: String,
    y_label: String,
}

impl eframe::App for App<'_> {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // lock released immediately; the copy keeps the audio callback fast
        let samples = self.audio.lock().unwrap().to_vec();

        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            let plot_height = ui.available_height() / 2.0 - ui.spacing().item_spacing.y;
            let history_secs = samples.len() as f64 / self.sample_rate as f64;

            fixed_plot("waveform")
                .height(plot_height)
                .x_axis_label("time (seconds)")
                .y_axis_label("amplitude")
                .default_x_bounds(-history_secs, 0.0)
                .default_y_bounds(-1.0, 1.0)
                .show(ui, |plot_ui| {
                    for line in waveform_lines("original", &samples, self.sample_rate) {
                        plot_ui.line(line);
                    }
                });

            let mut plot = fixed_plot("transformed")
                .height(plot_height)
                .x_axis_label(&self.x_label)
                .y_axis_label(&self.y_label);
            match &mut self.transform {
                Transform::Waveform(f) => {
                    let transformed = f(&samples, self.sample_rate);
                    let (x0, x1) = self
                        .x_range
                        .as_ref()
                        .map_or((-history_secs, 0.0), |r| (r.start, r.end));
                    let (y0, y1) = self
                        .y_range
                        .as_ref()
                        .map_or((-1.0, 1.0), |r| (r.start, r.end));
                    plot = plot.default_x_bounds(x0, x1).default_y_bounds(y0, y1);
                    plot.show(ui, |plot_ui| {
                        for line in waveform_lines("transformed", &transformed, self.sample_rate) {
                            plot_ui.line(line);
                        }
                    });
                }
                Transform::Points(f) => {
                    let points = f(&samples, self.sample_rate);
                    // an axis without a fixed range keeps automatic bounds
                    if let Some(r) = &self.x_range {
                        plot = plot.default_x_bounds(r.start, r.end);
                    }
                    if let Some(r) = &self.y_range {
                        plot = plot.default_y_bounds(r.start, r.end);
                    }
                    plot.show(ui, |plot_ui| {
                        let points: PlotPoints = points.iter().map(|(x, y)| [*x, *y]).collect();
                        plot_ui.line(Line::new("transformed", points));
                    });
                }
            }
        });

        // continuous rendering, the audio buffer changes permanently
        ctx.request_repaint();
    }
}

/// A plot with a fixed view: the live data moves, the cursor must not.
fn fixed_plot<'p>(id: &str) -> Plot<'p> {
    Plot::new(id)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .allow_double_click_reset(false)
        .allow_axis_zoom_drag(false)
}

/// The waveform as two plot lines (min/max envelope per pixel bucket, see
/// [`envelope`]), with x as seconds relative to now (`-history..0`).
fn waveform_lines<'l>(name: &str, samples: &[f32], sample_rate: f32) -> [Line<'l>; 2] {
    let history_secs = samples.len() as f64 / sample_rate as f64;
    let time_of = |sample_index: usize| sample_index as f64 / sample_rate as f64 - history_secs;

    let buckets = envelope(samples, MAX_POINTS);
    let upper: PlotPoints = buckets
        .iter()
        .map(|b| [time_of(b.start), b.max as f64])
        .collect();
    let lower: PlotPoints = buckets
        .iter()
        .map(|b| [time_of(b.start), b.min as f64])
        .collect();
    [
        Line::new(format!("{name} (upper)"), upper),
        Line::new(format!("{name} (lower)"), lower),
    ]
}
