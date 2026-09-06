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
//! Static frequency spectrum visualization: render `(frequency, magnitude)`
//! pairs to a PNG file or SVG string via [`Spectrum`].
//!
//! This crate does not compute spectra itself; pair it with an FFT crate such
//! as `spectrum-analyzer`. Individual frequencies can be highlighted in the
//! resulting chart, which is handy to verify that an expected peak is where
//! it should be. For real-time visualization see [`crate::dynamic`].

use crate::chart::{ensure_finite_and_non_empty, new_line_chart, write_png};
use crate::error::Error;
use charts_rs::{LineChart, Series};
use std::path::Path;

/// Builder that renders a frequency spectrum as an image.
///
/// Input is a list of `(frequency in Hz, magnitude)` pairs; it does not need
/// to be sorted. The x-axis is labeled with the frequencies, the y-axis
/// ranges from zero to the largest magnitude.
///
/// # Example
/// ```no_run
/// use audio_visualizer::spectrum::Spectrum;
///
/// let spectrum: Vec<(f32, f32)> = vec![(55.0, 0.1), (60.0, 0.9), (65.0, 0.2)];
/// Spectrum::new(&spectrum)
///     .highlight(60.0)
///     .write_png("spectrum.png")
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Spectrum<'a> {
    data: &'a [(f32, f32)],
    highlights: Vec<f32>,
    width: u32,
    height: u32,
    title: String,
}

impl<'a> Spectrum<'a> {
    /// Creates a spectrum visualization of `(frequency in Hz, magnitude)`
    /// pairs.
    #[must_use]
    pub const fn new(data: &'a [(f32, f32)]) -> Self {
        Self {
            data,
            highlights: vec![],
            width: 1400,
            height: 500,
            title: String::new(),
        }
    }

    /// Highlights the spectrum entry closest to the given frequency with a
    /// red marker. Can be called multiple times.
    #[must_use]
    pub fn highlight(mut self, frequency_hz: f32) -> Self {
        self.highlights.push(frequency_hz);
        self
    }

    /// Sets the image dimensions in pixels. Default: 1400x500.
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

    /// Renders the spectrum to an SVG string.
    pub fn to_svg(&self) -> Result<String, Error> {
        Ok(self.chart()?.svg()?)
    }

    /// Renders the spectrum and writes it as PNG file, creating missing
    /// parent directories.
    pub fn write_png(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        write_png(&self.chart()?, path.as_ref())
    }

    fn chart(&self) -> Result<LineChart, Error> {
        ensure_finite_and_non_empty(self.data.iter().flat_map(|(f, m)| [*f, *m]))?;

        let mut data = self.data.to_vec();
        data.sort_by(|a, b| a.0.total_cmp(&b.0));

        let x_labels = data.iter().map(|(f, _)| format_frequency(*f)).collect();
        let magnitudes = Series::new(
            "magnitude".to_string(),
            data.iter().map(|(_, m)| *m).collect(),
        );
        let mut series_list = vec![magnitudes];
        if !self.highlights.is_empty() {
            series_list.push(Series::new_nullable(
                "highlight".to_string(),
                highlight_series(&data, &self.highlights),
            ));
        }

        let mut chart = new_line_chart(series_list, x_labels, self.width, self.height, &self.title);
        chart.series_colors[1] = (255, 0, 0).into();
        let max_magnitude = data.iter().fold(0.0_f32, |acc, (_, m)| acc.max(*m));
        chart.y_axis_configs[0].axis_min = Some(0.0);
        chart.y_axis_configs[0].axis_max = Some(max_magnitude);
        Ok(chart)
    }
}

/// Overlay series that is `None` everywhere except a short segment around
/// the entry closest to each highlighted frequency. A segment of three
/// points guarantees visibility independent of the spectral resolution.
fn highlight_series(data: &[(f32, f32)], highlights: &[f32]) -> Vec<Option<f32>> {
    let mut overlay = vec![None; data.len()];
    for highlight in highlights {
        let nearest = data
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| (a.0 - highlight).abs().total_cmp(&(b.0 - highlight).abs()))
            .map(|(i, _)| i)
            .expect("data is non-empty");
        let range = nearest.saturating_sub(1)..=(nearest + 1).min(data.len() - 1);
        for i in range {
            overlay[i] = Some(data[i].1);
        }
    }
    overlay
}

/// Compact frequency label: no decimals for values that are (almost) whole.
fn format_frequency(frequency_hz: f32) -> String {
    if frequency_hz.fract().abs() < 0.01 {
        format!("{frequency_hz:.0}")
    } else {
        format!("{frequency_hz:.1}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::testutil::TEST_OUT_DIR;

    /// The old hardcoded example spectrum with a peak at 60 Hz.
    fn peak_60hz_spectrum() -> Vec<(f32, f32)> {
        [
            (0, 0.0),
            (10, 5.0),
            (20, 20.0),
            (30, 40.0),
            (40, 80.0),
            (50, 120.0),
            (55, 130.0),
            (60, 140.0),
            (65, 130.0),
            (70, 120.0),
            (80, 80.0),
            (90, 40.0),
            (100, 20.0),
            (110, 5.0),
            (120, 0.0),
            (130, 0.0),
        ]
        .iter()
        .map(|(f, m)| (*f as f32, *m))
        .collect()
    }

    #[test]
    fn highlight_marks_nearest_entries() {
        let data = peak_60hz_spectrum();
        let overlay = highlight_series(&data, &[61.0]);
        // nearest to 61 Hz is 60 Hz (index 7), plus one neighbor on each side
        assert_eq!(overlay[6], Some(130.0));
        assert_eq!(overlay[7], Some(140.0));
        assert_eq!(overlay[8], Some(130.0));
        assert_eq!(overlay.iter().filter(|v| v.is_some()).count(), 3);
    }

    #[test]
    fn rejects_nan() {
        assert!(matches!(
            Spectrum::new(&[(0.0, f32::NAN)]).to_svg(),
            Err(Error::InvalidData(_))
        ));
    }

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(
            Spectrum::new(&[]).to_svg(),
            Err(Error::InvalidData(_))
        ));
    }

    #[test]
    fn writes_png_file() {
        let data = peak_60hz_spectrum();
        Spectrum::new(&data)
            .highlight(60.0)
            .title("Spectrum with peak at 60 Hz")
            .write_png(format!("{TEST_OUT_DIR}/spectrum_60hz_peak.png"))
            .unwrap();
    }
}
