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
//! Static waveform visualization: render mono audio samples to a PNG file or
//! SVG string via [`Waveform`].
//!
//! Audio usually has far more samples than the image has horizontal pixels.
//! Rendering therefore reduces the samples to a min/max envelope per pixel
//! bucket first, so no peak is lost to naive downsampling and huge inputs
//! render fast. For real-time visualization see [`crate::dynamic`].

pub mod plotters_png_file;
pub mod png_file;

use crate::chart::{ensure_finite_and_non_empty, new_line_chart, write_png};
use crate::error::Error;
use charts_rs::{LineChart, Series};
use std::path::Path;

/// Upper bound of chart points; roughly one point per horizontal pixel of the
/// default image width.
const MAX_POINTS: usize = 1200;

/// Builder that renders mono audio samples as a waveform image.
///
/// Samples are expected as amplitudes in `[-1.0, 1.0]`, the usual DSP
/// convention; other symmetric ranges work too since the y-axis scales to the
/// data. For interleaved stereo data, split it with
/// [`crate::deinterleave_stereo`] first and render each channel separately.
///
/// # Example
/// ```no_run
/// use audio_visualizer::waveform::Waveform;
///
/// let samples: Vec<f32> = vec![0.0, 0.5, -0.5, 0.3];
/// Waveform::new(&samples)
///     .sample_rate(44100.0)
///     .write_png("waveform.png")
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Waveform<'a> {
    samples: &'a [f32],
    sample_rate: Option<f32>,
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
        Ok(self.chart()?.svg()?)
    }

    /// Renders the waveform and writes it as PNG file, creating missing
    /// parent directories.
    pub fn write_png(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        write_png(&self.chart()?, path.as_ref())
    }

    fn chart(&self) -> Result<LineChart, Error> {
        ensure_finite_and_non_empty(self.samples.iter().copied())?;

        let buckets = envelope(self.samples, MAX_POINTS);
        let x_labels = buckets.iter().map(|b| self.x_label(b.start)).collect();
        let mut upper = Series::new("upper".to_string(), buckets.iter().map(|b| b.max).collect());
        let mut lower = Series::new("lower".to_string(), buckets.iter().map(|b| b.min).collect());
        // Same palette slot: both envelope halves should look like one shape.
        upper.index = Some(0);
        lower.index = Some(0);

        let mut chart = new_line_chart(
            vec![upper, lower],
            x_labels,
            self.width,
            self.height,
            &self.title,
        );
        let max_abs = self.samples.iter().fold(0.0_f32, |acc, s| acc.max(s.abs()));
        let y_max = if max_abs == 0.0 { 1.0 } else { max_abs };
        chart.y_axis_configs[0].axis_min = Some(-y_max);
        chart.y_axis_configs[0].axis_max = Some(y_max);
        Ok(chart)
    }

    fn x_label(&self, sample_index: usize) -> String {
        self.sample_rate.map_or_else(
            || sample_index.to_string(),
            |rate| format!("{:.2}s", sample_index as f32 / rate),
        )
    }
}

/// One chart point per bucket of consecutive samples.
struct Bucket {
    /// Index of the bucket's first sample, used for the x-axis label.
    start: usize,
    min: f32,
    max: f32,
}

/// Reduces the samples to at most `max_points` min/max buckets.
fn envelope(samples: &[f32], max_points: usize) -> Vec<Bucket> {
    let bucket_len = samples.len().div_ceil(max_points);
    samples
        .chunks(bucket_len)
        .enumerate()
        .map(|(i, bucket)| {
            let (min, max) = bucket
                .iter()
                .fold((f32::MAX, f32::MIN), |(lo, hi), s| (lo.min(*s), hi.max(*s)));
            Bucket {
                start: i * bucket_len,
                min,
                max,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::testutil::TEST_OUT_DIR;

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
        let samples = (0..44100)
            .map(|i| (i as f32 / 44100.0 * 2.0 * std::f32::consts::PI * 100.0).sin())
            .collect::<Vec<_>>();
        Waveform::new(&samples)
            .sample_rate(44100.0)
            .title("100 Hz sine wave")
            .write_png(format!("{TEST_OUT_DIR}/waveform_sine_100hz.png"))
            .unwrap();
    }
}
