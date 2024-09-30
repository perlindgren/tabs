use std::fs::File;

use cpal::traits::{DeviceTrait, HostTrait};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::path::Path;
use std::time::Duration;
use symphonia::core::conv::IntoSample;

fn main() {
    let host = cpal::default_host();

    let device = host.default_output_device().unwrap();

    println!("Output device: {}", device.name().unwrap());

    let file = File::open(Path::new("./amazing_grace.mp3")).unwrap();
    let file2 = File::open(Path::new("./landskap_a_nameless_fool.mp3")).unwrap();
    play(&device, file, file2);
}

fn play(device: &cpal::Device, file: File, file2: File) {
    let source = Decoder::new(file).unwrap();
    let sample_rate = source.sample_rate();
    let channels = source.channels();
    let samples = source.convert_samples::<f32>();
    let mut samples_vec = vec![];
    for sample in samples {
        samples_vec.push(sample.into_sample());
    }

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(samples_vec.iter());
    std::thread::sleep(Duration::from_secs(10));
}
