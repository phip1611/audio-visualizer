//! Static spectrum analysis: print to PNG file.

use plotters::prelude::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub fn spectrum_static_plotters_png_visualize(
    frequency_spectrum: &BTreeMap<usize, f32>,
    directory: &str,
    filename: &str,
    normalize_to_median: bool,
) {
    // optionally normalize to median
    let median = if normalize_to_median {
        // find median and normalize to median
        let mut sorted_magnitudes = frequency_spectrum
            .values()
            .map(|x| *x)
            .collect::<Vec<f32>>();
        sorted_magnitudes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        sorted_magnitudes[frequency_spectrum.len() / 2]
    } else {
        0.0
    };

    let mut normalized_frequency_spectrum = BTreeMap::new();
    for (f, mag) in frequency_spectrum {
        if *mag < 0.0 {
            normalized_frequency_spectrum.insert(*f, 0.0);
        } else {
            normalized_frequency_spectrum.insert(*f, *mag - median);
        }
    }

    // find maximum for graphics scaling
    let mut max = 0.0;
    for (_fr, mag) in &normalized_frequency_spectrum {
        if *mag > max {
            max = *mag;
        }
    }

    let max_frequency = *normalized_frequency_spectrum
        .iter()
        .skip(normalized_frequency_spectrum.len() - 2)
        .last()
        .unwrap()
        .0;

    let mut path = PathBuf::new();
    path.push(directory);
    path.push(filename);

    let mut width = frequency_spectrum.len() as u32;
    if width < 300 {
        width = 300;
    }

    let mut height = 1000;
    if width < 1000 {
        height = (width as f32/0.8) as u32;
    }

    let root = BitMapBackend::new(&path, (width as u32, height)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption("y=f magnitudes of sample", ("sans-serif", 20).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            0.0..(max_frequency as f32), /*.log10()*/
            0.0..max as f32,
        )
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            // (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
            normalized_frequency_spectrum
                .iter()
                .into_iter()
                .map(|(frequency, magnitude)| ((*frequency as f32) /*.log10()*/, *magnitude)),
            &RED,
        ))
        .unwrap()
        .label("frequency magnitude")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TEST_OUT_DIR;

    #[test]
    fn test_visualize_sine_waves_spectrum_plotters() {
        let mut spectrum = BTreeMap::new();
        spectrum.insert(0, 0.0);
        spectrum.insert(10, 5.0);
        spectrum.insert(20, 20.0);
        spectrum.insert(30, 40.0);
        spectrum.insert(40, 80.0);
        spectrum.insert(50, 120.0);
        spectrum.insert(55, 130.0);
        spectrum.insert(60, 140.0);
        spectrum.insert(65, 130.0);
        spectrum.insert(70, 120.0);
        spectrum.insert(80, 80.0);
        spectrum.insert(90, 40.0);
        spectrum.insert(100, 20.0);
        spectrum.insert(110, 5.0);
        spectrum.insert(120, 0.0);
        spectrum.insert(130, 0.0);

        // Do FFT + get spectrum
        spectrum_static_plotters_png_visualize(
            &spectrum,
            TEST_OUT_DIR,
            "spectrum_60hz_peak_plotters_visualization.png",
            false,
        );
    }
}
