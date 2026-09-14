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
//! stream config, recording appends mono samples to a shared [`AudioBuffer`].
//!
//! Works cross-platform: Windows (WASAPI), Linux (ALSA), macOS (coreaudio).

use crate::Error;
use cpal::traits::{DeviceTrait, HostTrait};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

/// Callback size in frames to request when the stream config leaves the
/// buffer size to the device.
///
/// The visualization only moves when a callback delivers new samples, so
/// the callback rate caps the perceived frame rate regardless of how fast
/// the window renders. Device defaults are coarse: PipeWire's default
/// quantum is 1024 frames, i.e. ~47 callbacks/s at 48 kHz, which visibly
/// stutters on a 100 Hz display (one update every ~2 frames). 256 frames is
/// 5.3 ms at 48 kHz, enough for displays well beyond 144 Hz, and cheap: the
/// callback only appends to the ringbuffer.
const PREFERRED_BUFFER_FRAMES: u32 = 256;

/// The latest recorded samples together with the number of samples recorded
/// in total.
///
/// The total is what makes the visualization stable: it turns an index into
/// the ringbuffer into an absolute position in the audio stream, which does
/// not move when new samples arrive.
pub(crate) struct AudioBuffer {
    samples: AllocRingBuffer<f32>,
    total: u64,
}

impl AudioBuffer {
    /// Creates a buffer holding the latest `capacity` samples, pre-filled
    /// with silence so that the waveform covers the whole time axis right
    /// from the start.
    ///
    /// The pre-filled silence counts towards the total: the buffer is full
    /// from the very first frame, so the stream has to start `capacity`
    /// samples before the first recorded one. Starting the count at zero
    /// would make the stream position of the oldest buffered sample
    /// negative for as long as the recording is shorter than the buffer.
    ///
    /// `capacity` must be a power of two (ringbuffer requirement).
    pub(crate) fn new(capacity: usize) -> Self {
        let mut samples = AllocRingBuffer::new(capacity);
        samples.fill(0.0);
        Self {
            samples,
            total: capacity as u64,
        }
    }

    /// The buffered samples (oldest first) and the absolute stream position
    /// just past the newest one.
    pub(crate) fn snapshot(&self) -> (Vec<f32>, u64) {
        (self.samples.to_vec(), self.total)
    }

    fn extend(&mut self, samples: impl ExactSizeIterator<Item = f32>) {
        self.total += samples.len() as u64;
        self.samples.extend(samples);
    }
}

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

    /// Uses the given device with a mono input configuration if it offers
    /// one, otherwise with its default input configuration.
    ///
    /// Everything is visualized as mono anyway, so recording mono directly
    /// halves the data the device has to deliver and saves the downmix. For
    /// a device that only offers stereo, recording averages the two
    /// channels.
    pub fn from_device(dev: cpal::Device) -> Result<Self, Error> {
        let default = dev
            .default_input_config()
            .map_err(|e| Error::Audio(format!("no default input config: {e}")))?;
        let cfg = mono_config(&dev, &default).unwrap_or_else(|| default.config());
        Ok(Self { dev, cfg })
    }

    /// Uses the given device and stream configuration.
    ///
    /// A `buffer_size` of [`cpal::BufferSize::Default`] does not mean the
    /// device default: recording then asks for 256 frames per callback,
    /// which keeps the visualization moving every frame on high refresh
    /// rate displays, and only falls back to the device default if the
    /// device rejects that. A fixed size is used as given.
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
        latest_audio_data: Arc<Mutex<AudioBuffer>>,
    ) -> Result<cpal::Stream, Error> {
        let channels = self.cfg.channels;
        if channels != 1 && channels != 2 {
            return Err(Error::Audio(format!(
                "only mono or stereo input is supported, device has {channels} channels"
            )));
        }
        let is_mono = channels == 1;

        let build = |cfg: cpal::StreamConfig| {
            let latest_audio_data = latest_audio_data.clone();
            self.dev.build_input_stream(
                cfg,
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
        };

        if matches!(self.cfg.buffer_size, cpal::BufferSize::Default) {
            let preferred = cpal::StreamConfig {
                buffer_size: cpal::BufferSize::Fixed(PREFERRED_BUFFER_FRAMES),
                ..self.cfg
            };
            // Some devices only support their own period size and reject
            // this right here (e.g. "not in the supported range
            // 1024..=1024"); the default size then still records, just with
            // coarser updates.
            if let Ok(stream) = build(preferred) {
                return Ok(stream);
            }
        }
        build(self.cfg).map_err(|e| Error::Audio(format!("can't build input stream: {e}")))
    }
}

/// A mono configuration of `dev` with the sample rate of `default`, if the
/// device supports one.
///
/// Only `f32` is considered, because that is what recording asks the device
/// for.
fn mono_config(
    dev: &cpal::Device,
    default: &cpal::SupportedStreamConfig,
) -> Option<cpal::StreamConfig> {
    let sample_rate = default.sample_rate();
    dev.supported_input_configs()
        .ok()?
        .find(|cfg| {
            cfg.channels() == 1
                && cfg.sample_format() == cpal::SampleFormat::F32
                && (cfg.min_sample_rate()..=cfg.max_sample_rate()).contains(&sample_rate)
        })
        .map(|cfg| cfg.with_sample_rate(sample_rate).config())
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
