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
use crate::dynamic::live_input::setup_audio_input_loop;
use crate::dynamic::window_top_btm::visualize_minifb::{
    DEFAULT_H, DEFAULT_W, get_drawing_areas, setup_window,
};
use cpal::traits::StreamTrait;
use minifb::Key;
use plotters::chart::ChartContext;
use plotters::coord::cartesian::Cartesian2d;
use plotters::coord::types::RangedCoordf64;
use plotters::element::{Circle, EmptyElement, PathElement, Text};
use plotters::prelude::{BitMapBackend, IntoFont};
use plotters::series::{LineSeries, PointSeries};
use plotters::style::{BLACK, CYAN, RED};
use plotters_bitmap::bitmap_pixel::BGRXPixel;
use ringbuffer::{AllocRingBuffer, RingBufferExt};
use std::borrow::{Borrow, BorrowMut};
use std::mem::size_of;
use std::ops::Range;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub const SAMPLING_RATE: f64 = 44100.0;
pub const TIME_PER_SAMPLE: f64 = 1.0 / SAMPLING_RATE;
/// The number of latest samples that are kept for analysis in a ringbuffer.
/// Must be a power (ringbuffer requirement).
pub const AUDIO_SAMPLE_HISTORY_LEN: usize = (SAMPLING_RATE as usize * 5).next_power_of_two();
/// Smooth refresh rate on 144 Hz displays.
const REFRESH_RATE: f64 = 144.0;
const REFRESH_S: f64 = 1.0 / REFRESH_RATE;

pub mod pixel_buf;
pub mod visualize_minifb;

/// Parameter type for [`open_window_connect_audio`]. Describes how the audio data shall
/// be transformed, and thus, how it should be displayed in the lower part of the window.
///
/// The function is called every x milliseconds (refresh rate of window).
pub enum TransformFn<'a> {
    /// Synchronized x-axis with the original data. Useful for transformations on the
    /// waveform, such as a (lowpass) filter.
    ///
    /// Functions takes amplitude values and transforms them to a new amplitude value.
    Basic(fn(&[f32]) -> Vec<f32>),
    /// Use this, when the x-axis is different than for the original data. For example,
    /// if you want to display a spectrum.
    ///
    /// Functions takes amplitude values (and their index) and transforms them to a new
    /// (x,y)-pair. Takes a closure instead of a function, so that it can capture state.
    Complex(&'a dyn Fn(&[f32]) -> Vec<(f64, f64)>),
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
/// - `preferred_input_dev` Preferred audio input device. If None, it uses the default input device.
/// - `audio_data_transform_fn` See [`open_window_connect_audio`].
pub fn open_window_connect_audio(
    name: &str,
    preferred_height: Option<usize>,
    preferred_width: Option<usize>,
    preferred_x_range: Option<Range<f64>>,
    preferred_y_range: Option<Range<f64>>,
    x_desc: &str,
    y_desc: &str,
    preferred_input_dev: Option<cpal::Device>,
    audio_data_transform_fn: TransformFn,
) {
    let mut latest_audio_data = init_ringbuffer();
    let stream = setup_audio_input_loop(latest_audio_data.clone(), preferred_input_dev, SAMPLING_RATE as u32);
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
    );
    window.limit_update_rate(Some(Duration::from_secs_f64(REFRESH_S)));

    // GUI refresh loop; CPU-limited by "window.limit_update_rate"
    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        let (top_drawing_area, btm_drawing_area) =
            get_drawing_areas(pixel_buf.borrow_mut(), DEFAULT_W, DEFAULT_H);

        let top_chart = top_cs.clone().restore(&top_drawing_area);
        let btm_chart = btm_cs.clone().restore(&btm_drawing_area);

        // remove drawings from previous iteration (but keep axis etc)
        top_chart.plotting_area().fill(&BLACK).borrow();
        btm_chart.plotting_area().fill(&BLACK).borrow();

        // lock released immediately after oneliner
        let latest_audio_data = latest_audio_data.clone().lock().unwrap().to_vec();
        fill_chart_waveform_over_time(top_chart, &latest_audio_data);
        if let TransformFn::Basic(fnc) = audio_data_transform_fn {
            let data = fnc(&latest_audio_data);
            fill_chart_waveform_over_time(btm_chart, &data);
        } else if let TransformFn::Complex(fnc) = audio_data_transform_fn {
            let data = fnc(&latest_audio_data);
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
            .update_with_buffer(pixel_buf.borrow(), DEFAULT_W, DEFAULT_H)
            .unwrap();
    }
    stream.pause().unwrap();
}

/// Inits a ringbuffer on the heap and fills it with zeroes.
fn init_ringbuffer() -> Arc<Mutex<AllocRingBuffer<f32>>> {
    let mut buf = AllocRingBuffer::with_capacity(AUDIO_SAMPLE_HISTORY_LEN.next_power_of_two());
    buf.fill(0.0);
    Arc::new(Mutex::new(buf))
}

/// Fills the given chart with the waveform over time, from the past (left) to now/realtime (right).
fn fill_chart_complex_fnc(
    mut chart: ChartContext<BitMapBackend<BGRXPixel>, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    audio_data: Vec<(f64, f64)>,
) {
    // dedicated function; otherwise lifetime problems/compiler errors
   chart.draw_series(LineSeries::new(audio_data.into_iter(), &CYAN)).unwrap();
}

/// Fills the given chart with the waveform over time, from the past (left) to now/realtime (right).
fn fill_chart_waveform_over_time(
    mut chart: ChartContext<BitMapBackend<BGRXPixel>, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    audio_data: &[f32],
) {
    debug_assert_eq!(audio_data.len(), AUDIO_SAMPLE_HISTORY_LEN);
    const TIMESHIFT: f64 = AUDIO_SAMPLE_HISTORY_LEN as f64 * TIME_PER_SAMPLE;

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
            let timestamp = TIME_PER_SAMPLE * (i as f64) - TIMESHIFT;
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
            None,
            TransformFn::Basic(|x| x.to_vec()),
        );
    }
}
