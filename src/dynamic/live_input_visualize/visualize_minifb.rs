//! Helps to visualize audio data

use crate::dynamic::live_input_visualize::pixel_buf::PixelBuf;
use crate::dynamic::live_input_visualize::{
    AUDIO_SAMPLE_HISTORY_LEN, SAMPLING_RATE, TIME_PER_SAMPLE,
};
use minifb::{Window, WindowOptions};
use plotters::chart::{
    ChartBuilder, ChartContext, ChartState, LabelAreaPosition, SeriesLabelPosition,
};
use plotters::coord::cartesian::Cartesian2d;
use plotters::coord::types::RangedCoordf64;
use plotters::coord::Shift;
use plotters::drawing::{DrawingArea, IntoDrawingArea};
use plotters::style::{IntoFont, BLACK, WHITE};
use plotters_bitmap::bitmap_pixel::{BGRXPixel, RGBPixel};
use plotters_bitmap::BitMapBackend;
use std::borrow::{Borrow, BorrowMut};
use std::ops::Range;

/// Width of the window.
pub const DEFAULT_W: usize = 1280;
/// Height of the window.
pub const DEFAULT_H: usize = 720;

/// Initializes the [`minifb`] window and draws the initial grid into it.
/// It splits the drawing area into an upper chart and a lower chart. The
/// upper exists to show original audio data. The lower exists to show transformed
/// audio data, e.g. spectrum or lowpass filter.
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
///
/// # Returns
/// - window object
/// - chartstate of the upper chart
/// - chartstate of the lower chart
/// - the shared pixel buf
pub fn setup_window(
    name: &str,
    preferred_height: Option<usize>,
    preferred_width: Option<usize>,
    preferred_x_range: Option<Range<f64>>,
    preferred_y_range: Option<Range<f64>>,
    x_desc: &str,
    y_desc: &str,
) -> (
    Window,
    ChartState<Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    ChartState<Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    PixelBuf,
) {
    let mut window = Window::new(
        &String::from(name),
        DEFAULT_W,
        DEFAULT_H,
        WindowOptions::default(),
    )
    .unwrap();

    let height = preferred_height.unwrap_or(DEFAULT_H);
    let width = preferred_width.unwrap_or(DEFAULT_W);
    let x_range_top = -(AUDIO_SAMPLE_HISTORY_LEN as f64 * TIME_PER_SAMPLE)..0.0;
    let y_range_top = -1.0..1.01;
    let x_range_btm = preferred_x_range.unwrap_or(x_range_top.clone());
    let y_range_btm = preferred_y_range.unwrap_or(y_range_top.clone());

    // Buffer where we draw the Chart as bitmap into: we update the "minifb" window from it too
    let mut pixel_buf = PixelBuf(vec![0_u32; width * height]);

    let (top_drawing_area, btm_drawing_area) =
        get_drawing_areas(pixel_buf.borrow_mut(), width, height);

    let top_chart = draw_chart(
        top_drawing_area,
        x_range_top,
        y_range_top,
        "time (seconds)",
        "amplitude",
    );
    let btm_chart = draw_chart(btm_drawing_area, x_range_btm, y_range_btm, x_desc, y_desc);

    // unborrow "pixel_buf" again
    //drop(root_drawing_area);

    window
        .update_with_buffer(pixel_buf.borrow(), width, height)
        .unwrap();

    (window, top_chart, btm_chart, pixel_buf)
}

/// Returns two drawing areas, that together fill the whole window.
/// Upper: original audio data
/// Lower: transformed audio data
pub fn get_drawing_areas(
    pixel_buf: &mut [u8],
    width: usize,
    height: usize,
) -> (
    DrawingArea<BitMapBackend<BGRXPixel>, Shift>,
    DrawingArea<BitMapBackend<BGRXPixel>, Shift>,
) {
    // BGRXPixel format required by "minifb" (alpha, red, green, blue)
    let root_drawing_area = BitMapBackend::<BGRXPixel>::with_buffer_and_format(
        pixel_buf.borrow_mut(),
        (width as u32, height as u32),
    )
    .unwrap()
    .into_drawing_area();

    let (top_drawing_area, btm_drawing_area) =
        root_drawing_area.split_vertically((height / 2) as f64);
    (top_drawing_area, btm_drawing_area)
}

/// Draws the initial, empty into the dedicated drawing area.
/// Drops the drawing area, which is important to let this compile.
/// It's important that the chart gets returned as `ChartState`.
///
/// I don't understand it correctly, but it seems that the chart state is
/// a strategy by `plotter` to retain some state while not borrowing anything.
/// Furthermore this is more efficient, because axis etc. doesn't has to be
/// redrawn on incremental updates.
fn draw_chart<'a>(
    drawing_area: DrawingArea<BitMapBackend<BGRXPixel>, Shift>,
    x_range: Range<f64>,
    y_range: Range<f64>,
    x_desc: &'a str,
    y_desc: &'a str,
) -> ChartState<Cartesian2d<RangedCoordf64, RangedCoordf64>> {
    let mut chart = ChartBuilder::on(&drawing_area)
        // margin effects the distance to the border of the window of the chart
        .margin(10)
        .set_all_label_area_size(60)
        .build_cartesian_2d(x_range, y_range)
        .unwrap();

    chart
        .configure_mesh()
        .label_style(("sans-serif", 15).into_font().color(&WHITE))
        .x_desc(x_desc)
        .y_desc(y_desc)
        .x_labels(10)
        .y_labels(10)
        .axis_style(&WHITE)
        .draw()
        .unwrap();

    chart.into_chart_state()
}

#[cfg(test)]
mod tests {
    use super::*;
    use minifb::Key;

    #[test]
    fn test_minifb_window() {
        let (mut window, _, _, _) =
            super::setup_window("Test", None, None, -5.0..0.0, 0.0..5.01, "x-axis", "y-axis");
        while window.is_open() && !window.is_key_down(Key::Escape) {
            // REQUIRED to get keyboard and mouse events (such as close)
            window.update();
        }
    }
}