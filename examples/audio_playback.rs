use clap::Parser;
use rodio::{Decoder, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Example
///
/// cargo run --example audio_playback -- --speed 2.0
///
#[derive(Parser, Debug)]
struct Args {
    #[clap(
        short = 'p',
        long,
        help = "Input file path",
        default_value = "assets/landskap_a_nameless_fool.mp3"
    )]
    path: String,
    #[clap(short = 's', long, help = "Speed", default_value_t = 1.0f32)]
    speed: f32,
}

fn main() {
    let args = Args::parse();
    let path = &args.path;
    let path = Path::new(&path);

    let speed = args.speed;
    let stream_handle =
        rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");

    //  Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(File::open(path).unwrap());
    // Decode that sound file into a source
    let source = Decoder::new(file).unwrap().speed(speed);

    println!("Now playing: {}", args.path);
    let sink = Sink::connect_new(stream_handle.mixer());
    sink.append(source);

    std::thread::sleep(std::time::Duration::from_secs(5));
}
