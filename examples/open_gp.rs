use clap::Parser;
use scorelib::gp;
use std::{fs, io::Read, path::Path};
use tabs::{EADG, EADGBE};
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

    use tabs::{MidiNote, Note, Tuning};

    let mut strings: Vec<MidiNote> = vec![];

    for s in &track.strings {
        strings.push(MidiNote(s.1 as u32));
    }

    let strings: Vec<Note> = strings.iter().map(|item| (*item).into()).collect();

    let eadgbe = EADGBE {};
    let eadg = EADG {};
    let tuning: &dyn Tuning = if strings.as_slice() == eadgbe.tuning() {
        &eadgbe
    } else if strings.as_slice() == eadg.tuning() {
        &eadg
    } else {
        panic!("Unsupported tuning")
    };
    println!("Tuning: {:?}", tuning.tuning());
    let measure = track.measures.get(0).unwrap();
    let measure_1 = track.measures.get(1).unwrap();
    let voice = measure.voices.get(0).unwrap();
    let voice_1 = measure_1.voices.get(0).unwrap();
    for beat in &voice.beats {
        println!("--------------------------------------------------------------------------");
        println!("Beat: {:?}", beat.notes);
        println!("Duration: {:?}", beat.duration);
    }
    for beat in &voice_1.beats {
        println!("--------------------------------------------------------------------------");
        println!("Beat: {:?}", beat.notes);
        println!("Duration: {:?}", beat.duration);
    }
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