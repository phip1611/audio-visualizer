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
//! Audio visualization library for developers: quickly check audio samples
//! visually, e.g. while working on audio algorithms. It is not intended for
//! polished end-user visualizations.
//!
//! All functionality works on mono `f32` samples, typically amplitudes in
//! `[-1.0, 1.0]`. Split interleaved stereo data with [`deinterleave_stereo`]
//! first.
//!
//! - **Static images**: [`waveform::Waveform`] renders samples as
//!   PNG file or SVG string; [`spectrum`] does the same for frequency spectra.
//! - **Live visualization**: [`live`] records audio from an input device
//!   and shows the waveform plus a custom transformation (e.g. lowpass filter
//!   or spectrum) in a real-time GUI window.

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    // clippy::restriction,
    // clippy::pedantic
)]
// now allow a few rules which are denied by the above statement
// --> they are ridiculous and not necessary
#![allow(
    clippy::suboptimal_flops,
    clippy::redundant_pub_crate,
    clippy::fallible_impl_from,
    clippy::multiple_crate_versions
)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]

pub mod dynamic;
pub mod live;
pub mod spectrum;
pub mod waveform;

mod chart;
mod error;
#[cfg(test)]
mod tests;

pub use error::Error;

/// Splits interleaved stereo samples (left, right, left, right, ...) into a
/// left and a right channel vector.
///
/// # Panics
/// Panics if the number of samples is odd.
pub fn deinterleave_stereo(samples: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let (pairs, rest) = samples.as_chunks::<2>();
    assert!(
        rest.is_empty(),
        "stereo data must have an even number of samples"
    );
    pairs.iter().map(|[l, r]| (*l, *r)).unzip()
}
