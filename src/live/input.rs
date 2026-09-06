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
//! Audio recording via [`cpal`]: [`AudioInput`] selects an input device and
//! stream config, recording appends mono samples to a shared ringbuffer.
//!
//! Works cross-platform: Windows (WASAPI), Linux (ALSA), macOS (coreaudio).

use crate::Error;
use cpal::traits::{DeviceTrait, HostTrait};
use ringbuffer::AllocRingBuffer;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

/// The audio input device and stream configuration used for recording.
///
/// The caller must be certain that the config works for the given device on
/// the current platform; [`AudioInput::default_device`] and
/// [`AudioInput::from_device`] pick configs that do.
pub struct AudioInput {
    dev: cpal::Device,
    cfg: cpal::StreamConfig,
}

impl AudioInput {
    /// Uses the system default input device with its default configuration.
    pub fn default_device() -> Result<Self, Error> {
        let host = cpal::default_host();
        let dev = host.default_input_device().ok_or_else(|| {
            Error::Audio(format!(
                "no default audio input device found for host {}",
                host.id().name()
            ))
        })?;
        Self::from_device(dev)
    }

    /// Uses the given device with its default input configuration.
    pub fn from_device(dev: cpal::Device) -> Result<Self, Error> {
        let cfg = dev
            .default_input_config()
            .map_err(|e| Error::Audio(format!("no default input config: {e}")))?
            .config();
        Ok(Self { dev, cfg })
    }

    /// Uses the given device and stream configuration.
    #[must_use]
    pub const fn new(dev: cpal::Device, cfg: cpal::StreamConfig) -> Self {
        Self { dev, cfg }
    }

    /// All available input devices of the default host, sorted by name.
    pub fn devices() -> Result<Vec<(String, cpal::Device)>, Error> {
        let host = cpal::default_host();
        let mut devs: Vec<(String, cpal::Device)> = host
            .input_devices()
            .map_err(|e| Error::Audio(format!("can't enumerate input devices: {e}")))?
            .map(|dev| (dev.to_string(), dev))
            .collect();
        devs.sort_by(|(n1, _), (n2, _)| n1.cmp(n2));
        Ok(devs)
    }

    /// The input device.
    #[must_use]
    pub const fn device(&self) -> &cpal::Device {
        &self.dev
    }

    /// The stream configuration.
    #[must_use]
    pub const fn config(&self) -> &cpal::StreamConfig {
        &self.cfg
    }

    /// Builds an input stream that continuously appends the recorded audio
    /// to `latest_audio_data` as mono samples (stereo is averaged to mono).
    ///
    /// The stream still has to be started with
    /// [`cpal::traits::StreamTrait::play`] and records until dropped.
    pub(crate) fn build_stream(
        &self,
        latest_audio_data: Arc<Mutex<AllocRingBuffer<f32>>>,
    ) -> Result<cpal::Stream, Error> {
        let channels = self.cfg.channels;
        if channels != 1 && channels != 2 {
            return Err(Error::Audio(format!(
                "only mono or stereo input is supported, device has {channels} channels"
            )));
        }
        let is_mono = channels == 1;

        self.dev
            .build_input_stream(
                // Even if the supported configs claim that an input device
                // supports a fixed buffer size, ALSA and WASAPI tend to fail
                // with unclear errors. The default buffer size is the only
                // variant working reliably on all platforms and still gives
                // a good enough latency (~10ms on Windows, ~6ms on ALSA).
                self.cfg,
                move |data: &[f32], _info| {
                    let mut audio_buf = latest_audio_data.lock().unwrap();
                    if is_mono {
                        audio_buf.extend(data.iter().copied());
                    } else {
                        // interleaving for stereo is LRLR (de-facto standard)
                        let (pairs, _) = data.as_chunks::<2>();
                        audio_buf.extend(pairs.iter().map(|[l, r]| (l + r) / 2.0));
                    }
                },
                |err| eprintln!("audio stream error: {err:#?}"),
                None,
            )
            .map_err(|e| Error::Audio(format!("can't build input stream: {e}")))
    }
}

impl Debug for AudioInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioInput")
            .field("dev", &self.dev.to_string())
            .field("cfg", &self.cfg)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_input_devs() {
        dbg!(
            AudioInput::devices()
                .unwrap()
                .iter()
                .map(|(n, d)| (n, d.default_input_config()))
                .collect::<Vec<_>>()
        );
    }
}
