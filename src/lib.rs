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
//! Super basic and simple audio visualization library which is especially useful for developers to
//! visually check audio samples, e.g. by waveform or spectrum. (So far) this library is not
//! capable of doing nice visualizations for end users. Contributions are welcome.

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
    clippy::fallible_impl_from
)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::all)]

pub mod spectrum;
pub mod waveform;

pub mod dynamic;
#[cfg(test)]
mod tests;
pub mod util;

/// Describes the interleavement of audio data if
/// it is not mono but stereo.
#[derive(Debug, Copy, Clone)]
pub enum ChannelInterleavement {
    /// Stereo samples of one vector of audio data are alternating: left, right, left, right
    LRLR,
    /// Stereo samples of one vector of audio data are ordered like: left, left, ..., right, right
    /// In this case the length must be a multiple of 2.
    LLRR,
}

impl ChannelInterleavement {
    pub const fn is_lrlr(&self) -> bool {
        matches!(self, Self::LRLR)
    }
    pub const fn is_lllrr(&self) -> bool {
        matches!(self, Self::LLRR)
    }
    /// Transforms the interleaved data into two vectors.
    /// Returns a tuple. First/left value is left channel, second/right value is right channel.
    pub fn to_channel_data(&self, interleaved_data: &[i16]) -> (Vec<i16>, Vec<i16>) {
        let mut left_data = vec![];
        let mut right_data = vec![];

        if self.is_lrlr() {
            let mut is_left = true;
            for sample in interleaved_data {
                if is_left {
                    left_data.push(*sample);
                } else {
                    right_data.push(*sample)
                }
                is_left = !is_left;
            }
        } else {
            let n = interleaved_data.len();
            for sample_i in interleaved_data.iter().take(n / 2).copied() {
                left_data.push(sample_i);
            }
            for sample_i in interleaved_data.iter().skip(n / 2).copied() {
                right_data.push(sample_i);
            }
        }

        (left_data, right_data)
    }
}

/// Describes the number of channels of an audio stream.
#[derive(Debug, Copy, Clone)]
pub enum Channels {
    Mono,
    Stereo(ChannelInterleavement),
}

impl Channels {
    pub const fn is_mono(&self) -> bool {
        matches!(self, Self::Mono)
    }

    pub const fn is_stereo(&self) -> bool {
        matches!(self, Self::Stereo(_))
    }

    pub fn stereo_interleavement(&self) -> ChannelInterleavement {
        match self {
            Self::Stereo(interleavmement) => *interleavmement,
            _ => panic!("Not stereo"),
        }
    }
}
