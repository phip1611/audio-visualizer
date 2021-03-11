# Rust library: audio-visualizer

So far this library is really simple and mainly targets developers that develop audio algorithms. With this library
you can easily display your current audio data/waveform/spectrum and check if everything looks good. Functionality 
is really limited and follow the KISS (keep it simple, stupid) principle. Code contributions are very welcome!

#### Example of a waveform
![Example visualization of a waveform](png_waveform_example.png "Example visualization of a waveform")

#### Example of a spectrum
![Example visualization of a spectrum (0-140hz)](plotters_spectrum_example.png "Example visualization of a spectrum (0-140hz)")

## Covered Functionality
- **waveform**
  - **dynamic**: live visualization to audio input stream
    - [ ] TODO: code contributions are welcome
  - **static**: visualization of a single, static sample 
    - [x] very basic PNG output
    - [x] PNG output with basic axes/labels using https://crates.io/crates/plotters
    - [ ] TODO fancy static output
    
- **spectrum**
    - **dynamic**: visualization of a single, static sample
        - [ ] TODO: code contributions are welcome
    - **static**: visualization of a single, static sample
      - [x] very basic PNG output with the option to highlight specific frequencies
      - [x] PNG output with basic axes/labels using https://crates.io/crates/plotters
      - [ ] TODO fancy static output

## Example code
```rust
use minimp3::{Decoder as Mp3Decoder, Frame as Mp3Frame, Error as Mp3Error};
use audio_visualizer::ChannelInterleavement;
use audio_visualizer::Channels;
use audio_visualizer::waveform::staticc::png_file::visualize;
use std::path::PathBuf;
use std::fs::File;

/// Example that reads MP3 audio data and prints the waveform to a PNG file.
fn main() {
    let mut path = PathBuf::new();
    path.push("test/samples");
    path.push("sample_1.mp3");
    let mut decoder = Mp3Decoder::new(File::open(path).unwrap());

    let mut lrlr_mp3_samples = vec![];
    loop {
        match decoder.next_frame() {
            Ok(Mp3Frame { data: samples_of_frame, .. }) => {
                for sample in samples_of_frame {
                    lrlr_mp3_samples.push(sample);
                }
            }
            Err(Mp3Error::Eof) => break,
            Err(e) => panic!("{:?}", e),
        }
    }

    // library function
    visualize(
        &lrlr_mp3_samples,
        Channels::Stereo(ChannelInterleavement::LRLR),
        "src/test/out",
        "sample_1_waveform.png"
    );
}
```
