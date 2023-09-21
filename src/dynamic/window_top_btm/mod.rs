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
//! This module provides the functionality to display a GUI window, where the upper
//! half shows the real-time recorded audio data whereas the lower half shows a
//! diagram of transformed data, such as a lowpass filter or a a frequency spectrum.
//!
//! It uses the [`minifb`] crate to display GUI windows.
use crate::dynamic::live_input::{setup_audio_input_loop, AudioDevAndCfg};
use crate::dynamic::window_top_btm::visualize_minifb::{
    get_drawing_areas, setup_window, DEFAULT_H, DEFAULT_W,
};
use cpal::traits::StreamTrait;

use minifb::Key;
use plotters::chart::ChartContext;
use plotters::coord::cartesian::Cartesian2d;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::BitMapBackend;
use plotters::series::LineSeries;
use plotters::style::{BLACK, CYAN};
use plotters_bitmap::bitmap_pixel::BGRXPixel;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::borrow::{Borrow, BorrowMut};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Smooth refresh rate on 144 Hz displays.
const REFRESH_RATE: f64 = 144.0;
const REFRESH_S: f64 = 1.0 / REFRESH_RATE;

pub mod pixel_buf;
pub mod visualize_minifb;

/// Parameter type for [`open_window_connect_audio`]. Describes how the audio data shall
/// be transformed, and thus, how it should be displayed in the lower part of the window.
///
/// The function is called every x milliseconds (refresh rate of window).
///
/// This works cross-platform (Windows, MacOS, Linux).
#[allow(missing_debug_implementations)]
pub enum TransformFn<'a> {
    /// Synchronized x-axis with the original data. Useful for transformations on the
    /// waveform, such as a (lowpass) filter.
    ///
    /// Functions takes amplitude values and transforms them to a new amplitude value.
    /// It gets the sampling rate as second argument.
    Basic(fn(&[f32], f32) -> Vec<f32>),
    /// Use this, when the x-axis is different than for the original data. For example,
    /// if you want to display a spectrum.
    ///
    /// Functions takes amplitude values (and their index) and transforms them to a new
    /// (x,y)-pair. Takes a closure instead of a function, so that it can capture state.
    /// It gets the sampling rate as second argument.
    #[allow(clippy::complexity)]
    Complex(&'a dyn Fn(&[f32], f32) -> Vec<(f64, f64)>),
}

/// Starts the audio recording via `cpal` on the given audio device (or the default input device),
/// opens a GUI window and displays two graphs. The upper graph is the latest audio input as
/// wave form (live/real time). The lower graph can be customized, to show for example a
/// spectrum or the lowpassed data.
///
/// This operation is blocking. It returns, when the GUI window is closed.
///
/// **This operation is expensive and will be very laggy in "Debug" builds!**
///
/// # Parameters
/// - `name` Name of the GUI window
/// - `preferred_height` Preferred height of GUI window. Default is [`DEFAULT_H`].
/// - `preferred_width` Preferred height of GUI window. Default is [`DEFAULT_W`].
/// - `preferred_x_range` Preferred range for the x-axis of the lower (=custom) diagram.
///                       If no value is present, the same value as for the upper diagram is used.
/// - `preferred_y_range` Preferred range for the y-axis of the lower (=custom) diagram.
///                       If no value is present, the same value as for the upper diagram is used.
/// - `x_desc` Description for the x-axis of the lower (=custom) diagram.
/// - `y_desc` Description for the y-axis of the lower (=custom) diagram.
/// - `preferred_input_dev` See [`AudioDevAndCfg`].
/// - `audio_data_transform_fn` See [`open_window_connect_audio`].
#[allow(clippy::too_many_arguments)]
pub fn open_window_connect_audio(
    name: &str,
    preferred_height: Option<usize>,
    preferred_width: Option<usize>,
    preferred_x_range: Option<Range<f64>>,
    preferred_y_range: Option<Range<f64>>,
    x_desc: &str,
    y_desc: &str,
    input_dev_and_cfg: AudioDevAndCfg,
    audio_data_transform_fn: TransformFn,
) {
    let sample_rate = input_dev_and_cfg.cfg().sample_rate.0 as f32;
    let latest_audio_data = init_ringbuffer(sample_rate as usize);
    let audio_buffer_len = latest_audio_data.lock().unwrap().len();
    let stream = setup_audio_input_loop(latest_audio_data.clone(), input_dev_and_cfg);
    // This will be 1/44100 or 1/48000; the two most common sampling rates.
    let time_per_sample = 1.0 / sample_rate as f64;

    // start recording; audio will be continuously stored in "latest_audio_data"
    stream.play().unwrap();
    let (mut window, top_cs, btm_cs, mut pixel_buf) = setup_window(
        name,
        preferred_height,
        preferred_width,
        preferred_x_range,
        preferred_y_range,
        x_desc,
        y_desc,
        audio_buffer_len,
        time_per_sample,
    );
    window.limit_update_rate(Some(Duration::from_secs_f64(REFRESH_S)));

    // GUI refresh loop; CPU-limited by "window.limit_update_rate"
    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        let (top_drawing_area, btm_drawing_area) = get_drawing_areas(
            pixel_buf.borrow_mut(),
            preferred_width.unwrap_or(DEFAULT_W),
            preferred_height.unwrap_or(DEFAULT_H),
        );

        let top_chart = top_cs.clone().restore(&top_drawing_area);
        let btm_chart = btm_cs.clone().restore(&btm_drawing_area);

        // remove drawings from previous iteration (but keep axis etc)
        top_chart.plotting_area().fill(&BLACK).borrow();
        btm_chart.plotting_area().fill(&BLACK).borrow();

        // lock released immediately after oneliner
        let latest_audio_data = latest_audio_data.clone().lock().unwrap().to_vec();
        fill_chart_waveform_over_time(
            top_chart,
            &latest_audio_data,
            time_per_sample,
            audio_buffer_len,
        );
        if let TransformFn::Basic(fnc) = audio_data_transform_fn {
            let data = fnc(&latest_audio_data, sample_rate);
            fill_chart_waveform_over_time(btm_chart, &data, time_per_sample, audio_buffer_len);
        } else if let TransformFn::Complex(fnc) = audio_data_transform_fn {
            let data = fnc(&latest_audio_data, sample_rate);
            fill_chart_complex_fnc(btm_chart, data);
        } else {
            // required for compilation
            drop(btm_chart);
            panic!("invalid transform fn variant");
        }

        // make sure that "pixel_buf" is not borrowed longer
        drop(top_drawing_area);
        drop(btm_drawing_area);

        // REQUIRED to call on of the .update*()-methods, otherwise mouse and keyboard events
        // are not updated
        //
        // Update() also does the rate limiting/set the thread to sleep if not enough time
        //  sine the last refresh happened
        window
            .update_with_buffer(
                pixel_buf.borrow(),
                preferred_width.unwrap_or(DEFAULT_W),
                preferred_height.unwrap_or(DEFAULT_H),
            )
            .unwrap();
    }
    stream.pause().unwrap();
}

/// Inits a ringbuffer on the heap and fills it with zeroes.
fn init_ringbuffer(sampling_rate: usize) -> Arc<Mutex<AllocRingBuffer<f32>>> {
    // Must be a power (ringbuffer requirement).
    let mut buf = AllocRingBuffer::new((5 * sampling_rate).next_power_of_two());
    buf.fill(0.0);
    Arc::new(Mutex::new(buf))
}

/// Fills the given chart with the waveform over time, from the past (left) to now/realtime (right).
fn fill_chart_complex_fnc(
    mut chart: ChartContext<BitMapBackend<BGRXPixel>, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    audio_data: Vec<(f64, f64)>,
) {
    // dedicated function; otherwise lifetime problems/compiler errors
    chart
        .draw_series(LineSeries::new(audio_data, &CYAN))
        .unwrap();
}

/// Fills the given chart with the waveform over time, from the past (left) to now/realtime (right).
fn fill_chart_waveform_over_time(
    mut chart: ChartContext<BitMapBackend<BGRXPixel>, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    audio_data: &[f32],
    time_per_sample: f64,
    audio_history_buf_len: usize,
) {
    debug_assert_eq!(audio_data.len(), audio_history_buf_len);
    let timeshift = audio_history_buf_len as f64 * time_per_sample;

    // calculate timestamp of each index (x coordinate)
    let data_iter = audio_data
        .iter()
        .enumerate()
        // Important to reduce the calculation complexity by reducing the number of elements,
        // because drawing tens of thousands of points into the diagram is very expensive.
        //
        // If we skip too many elements, animation becomes un-smooth.... 4 seems to be sensible
        // due to tests by me.
        .filter(|(i, _)| *i % 4 == 0)
        .map(|(i, amplitude)| {
            let timestamp = time_per_sample * (i as f64) - timeshift;
            // Values for amplitude in interval [-1.0; 1.0]
            (timestamp, (*amplitude) as f64)
        });

    // Draws all points as a line of connected points.
    // LineSeries is reasonable efficient for the big workload, but still very expensive..
    // (4-6ms in release mode on my intel i5 10th generation)
    chart
        .draw_series(LineSeries::new(data_iter, &CYAN))
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_record_live_audio_and_visualize() {
        open_window_connect_audio(
            "Test",
            None,
            None,
            None,
            None,
            "x-axis",
            "y-axis",
            AudioDevAndCfg::new(None, None),
            TransformFn::Basic(|vals, _| vals.to_vec()),
        );
    }
}
