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
//! This module enables to record audio and store the latest audio data in a synchronized
//! ringbuffer. See [`setup_audio_input_loop`].
//!
//! It uses the [`cpal`] crate to record audio.

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{BufferSize, Device, FrameCount, SampleRate, StreamConfig};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

/// Sets up audio recording with the [`cpal`] library on the given audio input device.
/// If no input device is given, it uses the default input device. Panics, if it not present.
///
/// Appends all audio data to the ringbuffer `latest_audio_data`.
pub fn setup_audio_input_loop(
    latest_audio_data: Arc<Mutex<AllocRingBuffer<f32>>>,
    preferred_input_dev: Option<cpal::Device>,
    sampling_rate: u32,
) -> cpal::Stream {

    let input = preferred_input_dev.unwrap_or_else(|| {
        let host = cpal::default_host();
        let input = host.default_input_device().unwrap_or_else(|| {
            panic!(
                "No default audio input device found for host {}",
                host.id().name()
            )
        });
        input
    });
    println!(
        "Using input device: {}",
        input.name().as_ref().map(|x| x.as_str()).unwrap_or("<unknown>")
    );

    let cfg = StreamConfig {
        // I do only mono analysis here
        channels: 1,
        sample_rate: SampleRate(sampling_rate),
        // the lower, the better. We store the data in a ringbuffer anyway.
        // In practise, the buffer size received by the audio backend is variable
        // (at least on ALSA) anyway.
        buffer_size: BufferSize::Fixed(128),
    };

    input
        .build_input_stream(
            &cfg,
            // this is pretty cool by "cpal"; we can use u16, i16 or f32 and
            // the type system does all the magic behind the scenes
            move |data: &[f32], _info| {
                let mut audio_buf = latest_audio_data.lock().unwrap();
                audio_buf.extend(data.iter().map(|x| *x));
            },
            |_err| {},
        )
        .unwrap()
}

/// Lists all input devices for [`cpal`]. Can be used to select a device for
/// [`setup_audio_input_loop`].
pub fn list_input_devs() -> Vec<(String, cpal::Device)> {
    let host = cpal::default_host();
    type DeviceName = String;
    let mut devs: Vec<(DeviceName, Device)> = host
        .input_devices()
        .unwrap()
        .map(|dev| {
            (
                dev.name()
                    .map(|x| x.clone())
                    .unwrap_or(String::from("<unknown>")),
                dev,
            )
        })
        .collect();
    devs.sort_by(|(n1, _), (n2, _)| n1.cmp(n2));
    devs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_input_devs() {
        dbg!(list_input_devs()
            .iter()
            .map(|(n, d)| (n, d.default_input_config()))
            .collect::<Vec<_>>());
    }
}