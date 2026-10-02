/*
MIT License

Copyright (c) 2026 Philipp Schuster

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
//! Static waveform visualization: render mono audio samples to a PNG file or
//! SVG string via [`Waveform`].

use crate::chart::{ensure_finite_and_non_empty, ensure_valid_y_range, write_png};
use crate::error::Error;
use charts_rs::{DEFAULT_FONT_FAMILY, measure_text_width_family};
use std::ops::Range;
use std::path::Path;

// Colors and font sizes of the charts-rs light theme, so that waveform and
// spectrum images look alike.
const PEAK_COLOR: &str = "#5470c6";
const GRID_COLOR: &str = "#e0e6f2";
const AXIS_COLOR: &str = "#6e7079";
const TITLE_COLOR: &str = "#464646";
const FONT_SIZE: f32 = 14.0;
const TITLE_FONT_SIZE: f32 = 18.0;

/// Minimum distances between two axis ticks in pixels, so that the labels
/// do not crowd.
const MIN_X_TICK_SPACING: f32 = 80.0;
const MIN_Y_TICK_SPACING: f32 = 50.0;

/// Builder that renders mono audio samples as a waveform image.
///
/// Samples are expected as amplitudes in `[-1.0, 1.0]`, the usual DSP
/// convention; other symmetric ranges work too since the y-axis fits the
/// data unless [`Self::y_range`] fixes it. For interleaved stereo data, split
/// it with [`crate::deinterleave_stereo`] first and render each channel
/// separately.
///
/// # What the image shows
///
/// An image is far narrower than the audio is long: one second at 44.1 kHz
/// is dozens of samples per pixel column already. A column therefore is a
/// filled bar from the smallest to the largest sample it covers, as in audio
/// editors such as Audacity - the shape is the peak amplitude over time, and
/// the oscillation within a column is deliberately not resolved.
///
/// Drawing a line through every n-th sample instead would undersample the
/// signal by orders of magnitude: peaks between two picked samples vanish,
/// and what is left aliases into a pattern that is not in the audio.
///
/// For real-time visualization see [`crate::live`].
///
/// # Example
/// ```no_run
/// use audio_visualizer::WaveformVisualizer;
///
/// let samples: Vec<f32> = vec![0.0, 0.5, -0.5, 0.3];
/// WaveformVisualizer::new(&samples)
///     .sample_rate(44100.0)
///     .write_png("waveform.png")
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Waveform<'a> {
    samples: &'a [f32],
    sample_rate: Option<f32>,
    y_range: Option<Range<f32>>,
    width: u32,
    height: u32,
    title: String,
}

impl<'a> Waveform<'a> {
    /// Creates a waveform visualization of the given mono samples.
    #[must_use]
    pub const fn new(samples: &'a [f32]) -> Self {
        Self {
            samples,
            sample_rate: None,
            y_range: None,
            width: 1400,
            height: 400,
            title: String::new(),
        }
    }

    /// Labels the x-axis with seconds instead of sample indices.
    #[must_use]
    pub const fn sample_rate(mut self, sample_rate_hz: f32) -> Self {
        self.sample_rate = Some(sample_rate_hz);
        self
    }

    /// Fixes the y-axis to the given range instead of fitting it to the
    /// data. Amplitudes outside the range are clipped to its bounds.
    ///
    /// This makes images of different signals comparable: with `-1.0..1.0`,
    /// every image shows the full scale of `[-1.0, 1.0]` samples.
    #[must_use]
    pub const fn y_range(mut self, range: Range<f32>) -> Self {
        self.y_range = Some(range);
        self
    }

    /// Sets the image dimensions in pixels. Default: 1400x400.
    #[must_use]
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets a title displayed above the chart.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Renders the waveform to an SVG string.
    pub fn to_svg(&self) -> Result<String, Error> {
        ensure_finite_and_non_empty(self.samples.iter().copied())?;
        let samples_per_unit = self.samples_per_unit()?;
        let y_range = self.y_axis_range()?;

        let top = if self.title.is_empty() { 12.0 } else { 40.0 };
        let plot_height = self.height as f32 - top - 32.0;
        let y_ticks = ticks(&y_range, plot_height / MIN_Y_TICK_SPACING, "");
        let mut label_width = 0.0_f32;
        for (_, label) in &y_ticks {
            let size = measure_text_width_family(DEFAULT_FONT_FAMILY, FONT_SIZE, label)?;
            label_width = label_width.max(size.width());
        }
        // Whole pixels, so that every column of the waveform is exactly one
        // pixel wide.
        let left = (label_width + 16.0).round();
        let plot = Plot {
            left,
            top,
            width: self.width as f32 - left - 24.0,
            height: plot_height,
            len: self.samples.len(),
            y_range,
        };
        let x_range = 0.0..self.samples.len() as f32 / samples_per_unit;
        let x_unit = if self.sample_rate.is_some() { "s" } else { "" };
        let x_ticks = ticks(&x_range, plot.width / MIN_X_TICK_SPACING, x_unit);

        let (width, height) = (self.width, self.height);
        let (right, bottom) = (plot.left + plot.width, plot.bottom());
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" font-family="{DEFAULT_FONT_FAMILY}" font-size="{FONT_SIZE}">"#
        );
        svg.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);
        if !self.title.is_empty() {
            svg.push_str(&format!(
                r#"<text x="{}" y="26" text-anchor="middle" font-size="{TITLE_FONT_SIZE}" fill="{TITLE_COLOR}">{}</text>"#,
                width as f32 / 2.0,
                escape_xml(&self.title),
            ));
        }
        // Grid, waveform and axis are aligned to pixels; anti-aliasing
        // would only blur them.
        svg.push_str(r#"<g shape-rendering="crispEdges">"#);
        for (value, _) in &y_ticks {
            let y = plot.y(*value);
            svg.push_str(&format!(
                r#"<line x1="{left}" x2="{right}" y1="{y}" y2="{y}" stroke="{GRID_COLOR}"/>"#
            ));
        }
        svg.push_str(&format!(
            r#"<path d="{}" fill="{PEAK_COLOR}"/>"#,
            self.peak_path(&plot)
        ));
        svg.push_str(&format!(
            r#"<line x1="{left}" x2="{right}" y1="{bottom}" y2="{bottom}" stroke="{AXIS_COLOR}"/>"#
        ));
        for (value, _) in &x_ticks {
            let x = plot.x(value * samples_per_unit);
            let tick_end = bottom + 5.0;
            svg.push_str(&format!(
                r#"<line x1="{x}" x2="{x}" y1="{bottom}" y2="{tick_end}" stroke="{AXIS_COLOR}"/>"#
            ));
        }
        svg.push_str("</g>");
        for (value, label) in &y_ticks {
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="end" fill="{AXIS_COLOR}">{label}</text>"#,
                left - 8.0,
                plot.y(*value) + FONT_SIZE * 0.35,
            ));
        }
        for (value, label) in &x_ticks {
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="middle" fill="{AXIS_COLOR}">{label}</text>"#,
                plot.x(value * samples_per_unit),
                bottom + 8.0 + FONT_SIZE,
            ));
        }
        svg.push_str("</svg>");
        Ok(svg)
    }

    /// Renders the waveform and writes it as PNG file, creating missing
    /// parent directories.
    pub fn write_png(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        write_png(&self.to_svg()?, path.as_ref())
    }

    /// One filled column per pixel from the smallest to the largest sample
    /// it covers, as a single SVG path.
    fn peak_path(&self, plot: &Plot) -> String {
        let buckets = envelope(self.samples, plot.width as usize);
        let mut path = String::new();
        for (i, bucket) in buckets.iter().enumerate() {
            let end = buckets.get(i + 1).map_or(self.samples.len(), |b| b.start);
            // The column also covers the step to the next column's first
            // sample. Otherwise, the columns of a steep signal float apart,
            // as each one only covers its own samples.
            let next = self.samples.get(end).copied();
            let min = next.map_or(bucket.min, |n| bucket.min.min(n));
            let max = next.map_or(bucket.max, |n| bucket.max.max(n));
            let (top, bottom) = (plot.y(max), plot.y(min));
            // At least one pixel high, so that silence remains visible.
            let pad = ((1.0 - (bottom - top)) / 2.0).max(0.0);
            push_rect(
                &mut path,
                plot.x(bucket.start as f32)..plot.x(end as f32),
                top - pad..bottom + pad,
            );
        }
        path
    }

    /// Samples per x-axis unit: per second with a sample rate, otherwise the
    /// axis counts samples.
    fn samples_per_unit(&self) -> Result<f32, Error> {
        match self.sample_rate {
            None => Ok(1.0),
            Some(rate) if rate.is_finite() && rate > 0.0 => Ok(rate),
            Some(rate) => Err(Error::InvalidData(format!(
                "sample rate {rate} must be finite and positive"
            ))),
        }
    }

    /// The fixed range, or the peak amplitude mirrored around zero.
    fn y_axis_range(&self) -> Result<Range<f32>, Error> {
        if let Some(range) = &self.y_range {
            ensure_valid_y_range(range)?;
            return Ok(range.clone());
        }
        let max_abs = self.samples.iter().fold(0.0_f32, |acc, s| acc.max(s.abs()));
        let y_max = if max_abs == 0.0 { 1.0 } else { max_abs };
        Ok(-y_max..y_max)
    }
}

/// Maps sample positions and amplitudes to pixels of the plot area, i.e.
/// the image without title and axis labels.
struct Plot {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    /// Number of samples, spanning the full width.
    len: usize,
    y_range: Range<f32>,
}

impl Plot {
    fn x(&self, sample_index: f32) -> f32 {
        self.left + sample_index / self.len as f32 * self.width
    }

    /// Amplitudes outside the y-axis range are clipped to its bounds.
    fn y(&self, amplitude: f32) -> f32 {
        let Range { start, end } = self.y_range;
        let clipped = amplitude.clamp(start, end);
        self.top + (end - clipped) / (end - start) * self.height
    }

    fn bottom(&self) -> f32 {
        self.top + self.height
    }
}

/// Ticks at round values, i.e. multiples of 1, 2 or 5 times a power of
/// ten, using the smallest such step that splits `range` into at most
/// `max_intervals` intervals. Returns the value and label of each tick.
///
/// Computed in f64: in f32, the step underflows to zero for amplitudes close
/// to zero, and the span of a wide range overflows.
fn ticks(range: &Range<f32>, max_intervals: f32, unit: &str) -> Vec<(f32, String)> {
    let (start, end) = (f64::from(range.start), f64::from(range.end));
    let raw_step = (end - start) / f64::from(max_intervals.max(1.0));
    // An empty or infinite range has no meaningful ticks.
    if !raw_step.is_finite() || raw_step <= 0.0 {
        return Vec::new();
    }
    let exponent = raw_step.log10().floor() as i32;
    let magnitude = 10_f64.powi(exponent);
    let (step, exponent) = [1.0, 2.0, 5.0]
        .into_iter()
        .map(|m| m * magnitude)
        .find(|step| *step >= raw_step)
        .map_or((10.0 * magnitude, exponent + 1), |step| (step, exponent));
    let decimals = (-exponent).max(0) as usize;
    // The tolerance keeps a tick on a bound that is a multiple of the step
    // except for rounding errors.
    let first = (start / step - 1e-3).ceil() as i64;
    let last = (end / step + 1e-3).floor() as i64;
    (first..=last)
        .map(|k| {
            let value = k as f64 * step;
            (value as f32, format!("{value:.decimals$}{unit}"))
        })
        .collect()
}

/// Appends a rectangle as closed subpath to SVG path data.
fn push_rect(path: &mut String, x: Range<f32>, y: Range<f32>) {
    path.push_str(&format!(
        "M{:.1} {:.1}H{:.1}V{:.1}H{:.1}Z",
        x.start, y.start, x.end, y.end, x.start
    ));
}

/// Escapes text for use as content of an SVG element.
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// One chart point or image column per bucket of consecutive samples.
///
/// A bucket keeps only its minimum and maximum; everything between them is
/// discarded. That is what makes the reduction lossy but peak-preserving.
pub(crate) struct Bucket {
    /// Index of the bucket's first sample, used for the x-axis position.
    pub(crate) start: usize,
    pub(crate) min: f32,
    pub(crate) max: f32,
}

/// Reduces the samples to one min/max bucket per pixel column, or to one
/// bucket per sample if there are fewer samples than columns.
///
/// The bucket length follows from the input length, which is what a static
/// image needs: the whole input is covered whatever its size. Bucket lengths
/// differ by at most one sample, so that all columns have the same width.
fn envelope(samples: &[f32], columns: usize) -> Vec<Bucket> {
    let count = columns.min(samples.len());
    let boundary = |i: usize| i * samples.len() / count;
    (0..count)
        .map(|i| bucket_of(boundary(i), &samples[boundary(i)..boundary(i + 1)]))
        .collect()
}

/// Like [`envelope`], but with a caller-chosen bucket length and without a
/// trailing bucket that is not completely filled.
///
/// A scrolling view needs both, because its input is a moving window over a
/// stream instead of a fixed slice: the bucket length must not change with
/// the window, and a partially filled bucket would summarize a different
/// number of samples on every frame. See `live::aligned_envelope` for the
/// invariant built on top of this.
pub(crate) fn envelope_exact(samples: &[f32], bucket_len: usize) -> Vec<Bucket> {
    samples
        .chunks_exact(bucket_len)
        .enumerate()
        .map(|(i, bucket)| bucket_of(i * bucket_len, bucket))
        .collect()
}

fn bucket_of(start: usize, samples: &[f32]) -> Bucket {
    let (min, max) = samples
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), s| (lo.min(*s), hi.max(*s)));
    Bucket { start, min, max }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::numeric_labels;
    use crate::tests::testutil::TEST_OUT_DIR;

    fn full_scale_sine() -> Vec<f32> {
        (0..44100)
            .map(|i| (i as f32 / 44100.0 * 2.0 * std::f32::consts::PI * 100.0).sin())
            .collect()
    }

    #[test]
    fn auto_y_axis_is_symmetric_around_zero() {
        let waveform = Waveform::new(&[0.0, 0.8, -0.3]);
        assert_eq!(waveform.y_axis_range().unwrap(), -0.8..0.8);
    }

    #[test]
    fn y_axis_is_labeled_at_round_values() {
        let svg = Waveform::new(&full_scale_sine())
            .sample_rate(44100.0)
            .y_range(-1.0..1.0)
            .to_svg()
            .unwrap();
        assert_eq!(numeric_labels(&svg), [-1.0, -0.5, 0.0, 0.5, 1.0]);
    }

    #[test]
    fn ticks_use_round_steps() {
        let labels = |range, max_intervals| {
            ticks(&range, max_intervals, "")
                .into_iter()
                .map(|(_, label)| label)
                .collect::<Vec<_>>()
        };
        assert_eq!(labels(-0.79..0.79, 6.0), ["-0.5", "0.0", "0.5"]);
        assert_eq!(labels(0.2..1.0, 5.0), ["0.2", "0.4", "0.6", "0.8", "1.0"]);
        assert_eq!(
            labels(0.0..352_800.0, 8.0),
            [
                "0", "50000", "100000", "150000", "200000", "250000", "300000", "350000"
            ]
        );
    }

    #[test]
    fn y_range_clips_amplitudes_to_the_plot() {
        let plot = Plot {
            left: 0.0,
            top: 10.0,
            width: 100.0,
            height: 50.0,
            len: 1,
            y_range: -0.5..0.5,
        };
        assert_eq!(plot.y(1.0), 10.0);
        assert_eq!(plot.y(0.0), 35.0);
        assert_eq!(plot.y(-1.0), 60.0);
    }

    #[test]
    fn rejects_invalid_sample_rate() {
        for rate in [0.0, -44100.0, f32::NAN, f32::INFINITY] {
            assert!(matches!(
                Waveform::new(&[0.0]).sample_rate(rate).to_svg(),
                Err(Error::InvalidData(_))
            ));
        }
    }

    #[test]
    fn renders_extreme_amplitudes() {
        for amplitude in [1e-44, f32::MAX] {
            assert!(Waveform::new(&[amplitude, -amplitude]).to_svg().is_ok());
        }
    }

    #[test]
    fn escapes_the_title() {
        let svg = Waveform::new(&[0.0]).title("L&R <mix>").to_svg().unwrap();
        assert!(svg.contains(">L&amp;R &lt;mix&gt;</text>"));
    }

    #[test]
    fn rejects_invalid_y_range() {
        for range in [1.0..0.0, 0.0..0.0, f32::NAN..1.0, 0.0..f32::INFINITY] {
            assert!(matches!(
                Waveform::new(&[0.0]).y_range(range).to_svg(),
                Err(Error::InvalidData(_))
            ));
        }
    }

    #[test]
    fn envelope_keeps_peaks() {
        let mut samples = vec![0.1_f32; 1000];
        samples[500] = -0.9;
        samples[501] = 0.9;
        let buckets = envelope(&samples, 10);
        assert_eq!(buckets.len(), 10);
        assert_eq!(buckets[5].min, -0.9);
        assert_eq!(buckets[5].max, 0.9);
    }

    #[test]
    fn envelope_covers_all_samples() {
        let samples: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let buckets = envelope(&samples, 4);
        let starts: Vec<_> = buckets.iter().map(|b| b.start).collect();
        assert_eq!(starts, [0, 2, 5, 7]);
        assert_eq!(buckets[3].max, 9.0);
        // Fewer samples than columns: one bucket per sample.
        assert_eq!(envelope(&samples, 100).len(), 10);
    }

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(
            Waveform::new(&[]).to_svg(),
            Err(Error::InvalidData(_))
        ));
    }

    #[test]
    fn rejects_nan() {
        assert!(matches!(
            Waveform::new(&[0.0, f32::NAN]).to_svg(),
            Err(Error::InvalidData(_))
        ));
    }

    #[test]
    fn writes_png_file() {
        Waveform::new(&full_scale_sine())
            .sample_rate(44100.0)
            .title("100 Hz sine wave")
            .write_png(format!("{TEST_OUT_DIR}/waveform_sine_100hz.png"))
            .unwrap();
    }
}
