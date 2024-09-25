use clap::Parser;
use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
#[derive(Parser, Debug)]
struct Args {
    #[clap(
        short = 'p',
        long,
        help = "Input file path",
        default_value = "./landskap_a_nameless_fool.wav"
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
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(File::open(path).unwrap());
    // Decode that sound file into a source
    let source = Decoder::new(file).unwrap().speed(0.5);

    println!("Now playing: {}", args.path);
    // Play the sound directly on the device
    stream_handle.play_raw(source.convert_samples()).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(20));
}
