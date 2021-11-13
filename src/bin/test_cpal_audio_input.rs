use std::thread::sleep;
use std::time::Duration;
use cpal::{BufferSize, SampleRate, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Small binary to test audio input across platforms (windows, mac, linux) in a fast way.
fn main() {
    for host in cpal::available_hosts() {
        println!("Host={:?}, input devices:", host);
        for dev in cpal::host_from_id(host).unwrap().input_devices().unwrap() {
            println!(
                "  {} - Supported input configs:",
                dev.name()
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("<unknown>"),
            );

            for cfg in dev.supported_input_configs().unwrap() {
                println!("    {:#?}", cfg);
            }
        }
    }

    let default_in = cpal::default_host().default_input_device().unwrap();
    let stream = default_in.build_input_stream::<f32, _, _>(&StreamConfig {
        channels: 2,
        sample_rate: SampleRate(48000),
        buffer_size: BufferSize::Default,
    },
    |data, _x| {
        println!("got data: {} samples", data.len());
    },
    |_e| {

    }).unwrap();
    stream.play().unwrap();
    sleep(Duration::from_secs(5));
    stream.pause().unwrap();
}
