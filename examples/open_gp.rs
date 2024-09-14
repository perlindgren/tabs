use clap::Parser;
use scorelib::gp;
use std::{fs, io::Read, path::Path};
#[derive(Parser, Debug)]
struct Args {
    #[clap(short = 'p', long, help = "Input file path")]
    path: String,
}

fn main() {
    println!("Hello");
    let args: Args = Args::parse();
    let mut song: gp::Song = gp::Song::default();
    let f = Path::new(&args.path);
    let ext = f.extension().unwrap().to_str().unwrap();
    println!("{}", ext);
    let f = fs::File::open(f).unwrap();
    let mut data: Vec<u8> = vec![];
    for b in f.bytes() {
        data.push(b.unwrap());
    }

    match ext {
        "gp3" => song.read_gp3(&data),
        "gp4" => song.read_gp4(&data),
        "gp5" => song.read_gp5(&data),
        _ => {
            panic!("Invalid file extension (currently only .gp3, .gp4, .gp5 are supported)")
        }
    }
    println!("Operned song {} by {}", song.name, song.artist);

    for (i, track) in song.tracks.clone().into_iter().enumerate() {
        println!("[{}]: {}", i, track.name);
    }

    println!("Pick track:");
    let choice = get_input();
    let track = song.tracks.get(choice).unwrap();
    println!("Picked track: {}", track.name);

    use tabs::{MidiNote, Note};

    let mut strings: Vec<MidiNote> = vec![];

    for s in &track.strings {
        strings.push(MidiNote(s.1 as u32));
    }

    let strings: Vec<Note> = strings.iter().map(|item| (*item).into()).collect();
    println!("Tuning: {:?}", strings);
}

use std::io::{stdin, stdout, Write};
fn get_input() -> usize {
    let mut s = String::new();
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    let s = s.strip_suffix("\n").unwrap();
    s.parse::<usize>().unwrap()
}
