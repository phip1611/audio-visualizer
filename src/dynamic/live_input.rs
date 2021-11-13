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
use cpal::Device;
use ringbuffer::AllocRingBuffer;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

/// Describes the audio input device that should be used and the config for the input stream.
/// Caller must be certain, that the config works for the given device on the current platform.
pub struct AudioDevAndCfg {
    /// The input device.
    dev: cpal::Device,
    /// Desired configuration for the input stream.
    cfg: cpal::StreamConfig,
}

impl AudioDevAndCfg {
    /// Creates an instance. If no device is passed, it falls back to the default input
    /// device of the system.
    pub fn new(
        preferred_dev: Option<cpal::Device>,
        preferred_cfg: Option<cpal::StreamConfig>,
    ) -> Self {
        let dev = preferred_dev.unwrap_or_else(|| {
            let host = cpal::default_host();
            host.default_input_device().unwrap_or_else(|| {
                panic!(
                    "No default audio input device found for host {}",
                    host.id().name()
                )
            })
        });
        let cfg = preferred_cfg.unwrap_or_else(|| dev.default_input_config().unwrap().config());
        Self { dev, cfg }
    }

    /// Getter for audio device.
    pub const fn dev(&self) -> &cpal::Device {
        &self.dev
    }

    /// Getter for audio input stream config.
    pub const fn cfg(&self) -> &cpal::StreamConfig {
        &self.cfg
    }
}

impl Debug for AudioDevAndCfg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioDevAndCfg")
            .field(
                "dev",
                &self
                    .dev
                    .name()
                    .as_ref()
                    .map(|x| x.as_str())
                    .unwrap_or("<unknown>"),
            )
            .field("cfg", &self.cfg)
            .finish()
    }
}

/// Sets up audio recording with the [`cpal`] library on the given audio input device.
/// If no input device is given, it uses the default input device. Panics, if it not present.
/// Returns the stream plus the chosen config for the device.
///
/// Appends all audio data to the ringbuffer `latest_audio_data`.
///
/// Works on Windows (WASAPI), Linux (ALSA) and MacOS (coreaudio).
pub fn setup_audio_input_loop(
    latest_audio_data: Arc<Mutex<AllocRingBuffer<f32>>>,
    audio_dev_and_cfg: AudioDevAndCfg,
) -> cpal::Stream {
    let dev = audio_dev_and_cfg.dev();
    let cfg = audio_dev_and_cfg.cfg();

    eprintln!(
        "Using input device '{}' with config: {:?}",
        dev.name()
            .as_ref()
            .map(|x| x.as_str())
            .unwrap_or("<unknown>"),
        cfg
    );

    assert!(
        cfg.channels == 1 || cfg.channels == 2,
        "only supports Mono or Stereo channels!"
    );

    if cfg.sample_rate.0 != 44100 && cfg.sample_rate.0 != 48000 {
        eprintln!(
            "WARN: sampling rate is {}, but the crate was only tested with 44,1/48khz.",
            cfg.sample_rate.0
        );
    }

    let is_mono = cfg.channels == 1;

    let stream = dev
        .build_input_stream(
            // This is not as easy as it might look. Even if the supported configs show, that a
            // input device supports a given fixed buffer size, ALSA but also WASAPI tend to
            // fail with unclear error messages. I found out, that using the default option is the
            // only variant that is working on all platforms (Windows, Mac, Linux). The buffer
            // size tends to be not as small as it would be optimal (for super low latency)
            // but is still good enough (for example ~10ms on Windows) or ~6ms on ALSA (in my
            // tests).
            audio_dev_and_cfg.cfg(),
            // this is pretty cool by "cpal"; we can use u16, i16 or f32 and
            // the type system does all the magic behind the scenes. f32 also works
            // on Windows (WASAPI), MacOS (coreaudio), and Linux (ALSA).
            move |data: &[f32], _info| {
                let mut audio_buf = latest_audio_data.lock().unwrap();
                // Audio buffer only contains Mono data
                if is_mono {
                    audio_buf.extend(data.iter().copied());
                } else {
                    // interleaving for stereo is LRLR (de-facto standard?)
                    audio_buf.extend(data.chunks_exact(2).map(|vals| (vals[0] + vals[1]) / 2.0))
                }
            },
            |err| {
                eprintln!("got stream error: {:#?}", err);
            },
        )
        .unwrap();

    stream
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
                dev.name().unwrap_or_else(|_| String::from("<unknown>")),
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
