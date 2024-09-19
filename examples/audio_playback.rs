use clap::Parser;
use std::fs;
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
}

fn main() {
    let args = Args::parse();
    let path = &args.path;
    let path = Path::new(&path);
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let file = fs::File::open(path).unwrap();
    println!("Now playing: {}", args.path);
    let playback = stream_handle
        .play_once(std::io::BufReader::new(file))
        .unwrap();
    playback.set_volume(0.5);

    playback.sleep_until_end();
}
