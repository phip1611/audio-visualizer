//! Static spectrum analysis: print to PNG file.

use crate::util::png::write_png_file_rgb_tuples;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub fn spectrum_static_png_visualize(
    frequency_spectrum: &BTreeMap<usize, f32>,
    directory: &str,
    filename: &str,
    highlighted_frequencies: &[f32],
    normalize_to_median: bool,
) {
    let image_width = 5000;
    let image_height = 3000;

    let mut rgb_img = vec![vec![(255, 255, 255); image_width]; image_height];

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

    let x_step = image_width as f64 / normalized_frequency_spectrum.len() as f64;
    let mut i = 0;
    for (frequency, mag) in normalized_frequency_spectrum {
        let mag = mag / max * image_height as f32;

        let x = (i as f64 * x_step) as usize;

        for j in 0..mag as usize {
            let mut color = (0, 0, 0);

            let highlight = highlighted_frequencies
                .iter()
                .any(|f| (frequency as f32 - *f).abs() < 5.0);
            if highlight {
                color = (255, 0, 0);
            }

            // make it wider
            if x > 2 && highlight {
                rgb_img[image_height - 1 - j][x - 1] = color;
                rgb_img[image_height - 1 - j][x - 2] = color;
            }
            rgb_img[image_height - 1 - j][x] = color;
        }
        i += 1;
    }

    let mut path = PathBuf::new();
    path.push(directory);
    path.push(filename);
    write_png_file_rgb_tuples(&path, &rgb_img);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TEST_OUT_DIR;

    #[test]
    fn test_visualize_sine_waves_spectrum() {
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
        spectrum_static_png_visualize(
            &spectrum,
            TEST_OUT_DIR,
            "spectrum_60hz_peak_basic_visualization.png",
            &[60.0],
            false,
        );
    }
}
