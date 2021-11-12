//! Super basic and simple audio visualization library which is especially useful for developers to
//! visually check audio samples, e.g. by waveform or spectrum. (So far) this library is not
//! capable of doing nice visualizations for end users. Contributions are welcome.

pub mod spectrum;
pub mod waveform;

// public for examples
pub mod util;
// public for examples
pub mod dynamic;
pub mod test_support;

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
    pub fn is_lrlr(&self) -> bool {
        match self {
            ChannelInterleavement::LRLR => true,
            _ => false,
        }
    }
    pub fn is_lllrr(&self) -> bool {
        match self {
            ChannelInterleavement::LLRR => true,
            _ => false,
        }
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
            for sample_i in 0..n / 2 {
                left_data.push(interleaved_data[sample_i]);
            }
            for sample_i in n / 2..n {
                right_data.push(interleaved_data[sample_i]);
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
    pub fn is_mono(&self) -> bool {
        match self {
            Channels::Mono => true,
            _ => false,
        }
    }

    pub fn is_stereo(&self) -> bool {
        match self {
            Channels::Stereo(_) => true,
            _ => false,
        }
    }

    pub fn stereo_interleavement(&self) -> ChannelInterleavement {
        match self {
            Channels::Stereo(interleavmement) => interleavmement.clone(),
            _ => panic!("Not stereo"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
