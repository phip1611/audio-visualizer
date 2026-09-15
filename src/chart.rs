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
//! Internal glue between the public visualization builders and [`charts_rs`]:
//! shared chart defaults and PNG file export.

use crate::Error;
use charts_rs::{LineChart, Series, svg_to_png};
use std::fs;
use std::ops::Range;
use std::path::Path;

/// Creates a line chart with the crate-wide look: no point symbols, no
/// legend, x labels directly at the data points.
pub(crate) fn new_line_chart(
    series_list: Vec<Series>,
    x_labels: Vec<String>,
    width: u32,
    height: u32,
    title: &str,
) -> LineChart {
    let mut chart = LineChart::new(series_list, x_labels);
    chart.width = width as f32;
    chart.height = height as f32;
    chart.title_text = title.to_string();
    chart.legend_show = Some(false);
    chart.series_symbol = None;
    chart.x_boundary_gap = Some(false);
    chart
}

/// Renders the chart as PNG and writes it to `path`, creating missing parent
/// directories.
pub(crate) fn write_png(chart: &LineChart, path: &Path) -> Result<(), Error> {
    let png = svg_to_png(&chart.svg()?)?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, png)?;
    Ok(())
}

/// Rejects empty input and non-finite values (NaN, infinity) with a
/// descriptive [`Error::InvalidData`].
pub(crate) fn ensure_finite_and_non_empty(
    values: impl IntoIterator<Item = f32>,
) -> Result<(), Error> {
    let mut empty = true;
    for value in values {
        empty = false;
        if !value.is_finite() {
            return Err(Error::InvalidData(format!(
                "input contains non-finite value {value}"
            )));
        }
    }
    if empty {
        return Err(Error::InvalidData("input is empty".to_string()));
    }
    Ok(())
}

/// Fixes the y-axis of `chart` to `range`.
///
/// charts-rs treats a fixed bound as a suggestion: it only applies one that
/// lies strictly outside the data. Handing it the exact data range therefore
/// changes nothing, and the automatic ticks take over, rounding the upper
/// bound up and snapping a positive lower bound to zero. Moving such a
/// bound outwards by one ulp makes it count, and is far too small to change
/// a label.
///
/// A lower bound of zero or below needs no nudge, since charts-rs leaves it
/// alone. Zero must not get one either: one ulp below zero is negative, and
/// the axis would show "-0".
pub(crate) fn set_y_range(chart: &mut LineChart, range: &Range<f32>) {
    let min = if range.start > 0.0 {
        range.start.next_down()
    } else {
        range.start
    };
    chart.y_axis_configs[0].axis_min = Some(min);
    chart.y_axis_configs[0].axis_max = Some(range.end.next_up());
}

/// All `<text>` contents of `svg` that parse as numbers. These are the
/// y-axis labels as long as the x labels carry a unit suffix.
#[cfg(test)]
pub(crate) fn numeric_labels(svg: &str) -> Vec<f32> {
    svg.split("<text")
        .skip(1)
        .filter_map(|s| s.split_once('>')?.1.split_once("</text>"))
        .filter_map(|(label, _)| label.trim().parse().ok())
        .collect()
}
