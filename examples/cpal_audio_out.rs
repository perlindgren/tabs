use std::{
    borrow::{Borrow, BorrowMut},
    fs::File,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample,
};
use std::path::Path;
use symphonia::core::{
    audio::SampleBuffer,
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    errors::Error,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

fn main() {
    let host = cpal::default_host();

    let device = host.default_output_device().unwrap();

    println!("Output device: {}", device.name().unwrap());

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);
    let file = File::open(Path::new("./amazing_grace.mp3")).unwrap();

    open_audio(file);
    run(&device, &config.into()).ok();
}

fn open_audio(file: File) -> Vec<f32> {
    let file = Box::new(file);
    let mss = MediaSourceStream::new(file, Default::default());
    let hint = Hint::new();
    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();
    //let decoder_opts: DecoderOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .unwrap();

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .expect("no supported audio tracks");

    let track_id = track.id;

    let dec_opts: DecoderOptions = Default::default();

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");
    let mut samples_vec = vec![];
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::ResetRequired) => {
                todo!()
            }
            Err(Error::IoError(s)) => match s.kind() {
                std::io::ErrorKind::UnexpectedEof => {
                    panic!("EOF");
                }
                _ => {
                    panic!();
                }
            },
            Err(err) => {
                panic!("{}", err);
            }
        };

        while !format.metadata().is_latest() {
            format.metadata().pop();
        }

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = decoded.spec();
                let sampling_rate = spec.rate;
                let samples_count = decoded.capacity();
                println!(
                    "Decoded {} samples at {}Hz sampling freq",
                    samples_count, sampling_rate
                );
                let mut sample_buf = SampleBuffer::<f32>::new(samples_count as u64, *spec);

                sample_buf.copy_planar_ref(decoded);

                println!("----------------FRAME------------");
                for sample in sample_buf.samples() {
                    samples_vec.push(sample.clone());
                }
            }
            Err(Error::DecodeError(_)) => {
                // Failed to decode due to malformed data, skip packet
                continue;
            }
            Err(err) => {
                break;
            }
        }
    }

    samples_vec
}

pub fn run(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error> {
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0;
    let mut next_value = move || 0.0;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}

fn write_data(output: &mut [f32], channels: usize, next_sample: &mut dyn FnMut() -> f32) {
    for frame in output.chunks_mut(channels) {
        let value: f32 = f32::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
