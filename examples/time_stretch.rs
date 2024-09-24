#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

use clap::Parser;
use heapless::spsc::*;
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

struct Packet(u8, Duration, f32);

const QUEUE_SIZE: usize = 8;
type Q = Queue<Packet, QUEUE_SIZE>;

type P = Producer<'static, Packet, QUEUE_SIZE>;

use log::*;
use scorelib::gp;
use std::{fs, io::Read, path::Path};
use tabs::{fret_chart::*, *};

#[derive(Parser, Debug)]
struct Args {
    #[clap(
        short = 'p',
        long,
        help = "Input tab file path",
        default_value = "amazing_grace.gp5"
    )]
    path: String,
    #[clap(
        short = 'a',
        long,
        help = "Input audio file path",
        default_value = "amazing_grace.mp3"
    )]
    audio_path: String,
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    trace!("env_logger started");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1920.0, 540.0]),
        // vsync: false,
        ..Default::default()
    };
    eframe::run_native(
        "Fret Test",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    fret_board: FretChart,
    looping: bool,
    time_instant: Instant,
    bpm: f32,
    start_instant: Instant,
    playing_audio: bool,
    paused: bool,
    last_paused: Instant,
    paused_time: Duration,
    note_by_note: bool,
    beat: f32,
    stretch_factor: f32,
    transport: Duration,
    tx: P,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
        println!("Opened song {} by {}", song.name, song.artist);

        for (i, track) in song.tracks.clone().into_iter().enumerate() {
            println!("[{}]: {}", i, track.name);
        }

        println!("Pick track:");
        let choice = get_input();
        let track = song.tracks.get(choice).unwrap();
        println!("Picked track: {}", track.name);

        let mut strings: Vec<MidiNote> = vec![];

        for s in &track.strings {
            strings.push(MidiNote(s.1 as u32));
        }

        let mut strings: Vec<Note> = strings.iter().map(|item| (*item).into()).collect();
        strings.reverse();

        let eadgbe = EADGBE {};
        let eadg = EADG {};
        println!("{:?}", eadgbe.tuning());
        println!("{:?}", strings.as_slice());
        let tuning: Rc<dyn Tuning> = if strings.as_slice() == eadgbe.tuning() {
            Rc::new(eadgbe)
        } else if strings.as_slice() == eadg.tuning() {
            Rc::new(eadg)
        } else {
            panic!("Unsupported tuning")
        };
        println!("Tuning: {:?}", tuning.tuning());
        let mut fret_notes = vec![];

        //insert two measures of silence
        //let mut current_time = 2.0;
        let mut current_time = 0.0;
        let tempo = song.tempo;
        println!("tempo {}", tempo);
        for measure in &track.measures {
            let voice = measure.voices.first().unwrap();
            for beat in &voice.beats {
                for note in &beat.notes {
                    let fret_note = FretNote::new(
                        (note.string - 1) as u8, //zero indexed...
                        note.value as u8,
                        current_time as f32,
                        None,
                        tuning.clone(),
                    );
                    fret_notes.push(fret_note);
                    current_time += 1.0 / beat.duration.value as f64;
                }
            }
        }

        let fret_notes = FretNotes(fret_notes);

        let path = &args.audio_path;
        let path = Path::new(&path);
        let file = fs::File::open(path).unwrap();
        println!("Now playing: {}", args.path);
        let spsc: &'static mut Q = {
            static mut SPSC: Q = Queue::new();
            #[allow(static_mut_refs)]
            unsafe {
                &mut SPSC
            }
        };
        let (tx, mut rx) = spsc.split();

        std::thread::spawn(move || {
            let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

            let mut paused = false;
            println!("now playing");
            let playback = stream_handle
                .play_once(std::io::BufReader::new(file))
                .unwrap();
            playback.set_volume(0.5);
            let mut playhead = playback.get_pos();
            loop {
                if playhead != playback.get_pos() {
                    playhead = playback.get_pos();
                }
                match rx.dequeue() {
                    Some(b) => {
                        if b.0 == 1 {
                            if !paused {
                                paused = true;
                                playback.pause()
                            }
                        }
                        if b.0 == 0 {
                            if paused {
                                paused = false;
                                playback.try_seek(b.1.div_f32(b.2)).ok();
                                playback.play()
                            }
                        }
                        playback.set_speed(b.2);
                    }
                    None => {}
                }
            }
        });

        Self {
            fret_board: FretChart::new(fret_notes),
            looping: false,
            time_instant: Instant::now(),
            bpm: tempo as f32,
            start_instant: Instant::now(),
            playing_audio: false,
            paused: false,
            last_paused: Instant::now(),
            paused_time: Duration::from_secs(0),
            note_by_note: false,
            beat: 0.0,
            stretch_factor: 1.0,
            transport: Duration::from_secs_f32(0.0),
            tx,
        }
    }
}
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let now = Instant::now();
            let since = now - self.time_instant;
            let one_sec = Duration::from_secs(1);
            //now - (self.start_instant + self.paused_time);
            let bpm = self.bpm;
            if !self.playing_audio {
                //start playback
                self.tx
                    .enqueue(Packet(0, self.transport, self.stretch_factor))
                    .ok();
                self.playing_audio = true;
            }
            let f = (one_sec.as_micros() / since.as_micros()) as u32;
            ui.label(format!("Freq: {:?}", f));
            ui.label(format!("Transport: {:?}", self.transport));
            ui.label(format!(
                "Beat {}, Pos {}",
                1 + self.beat as u32 % 4,
                self.beat as u32
            ));
            ui.label(format!("Frame Time: {:?}", since));
            if ui.checkbox(&mut self.looping, "looping").clicked() {
                trace!("something clicked, clip_rect {:?}", ui.clip_rect());
            }
            if ui
                .add(
                    egui::Slider::new(&mut self.stretch_factor, 0.0..=1.0)
                        .text("Time stretch factor"),
                )
                .changed()
            {
                self.tx
                    .enqueue(Packet(
                        if self.paused { 1 } else { 0 },
                        self.transport,
                        self.stretch_factor,
                    ))
                    .ok();
            };
            if ui.button("restart").clicked() {
                trace!("restart {:?}", ui.clip_rect());
                self.start_instant = Instant::now();
            }
            if ui
                .button(format!("note by note: {}", self.note_by_note))
                .clicked()
            {
                if self.note_by_note {
                    self.note_by_note = false;
                } else {
                    self.note_by_note = true;
                }
            }
            self.time_instant = now;
            if self.paused {
                // Note detection to be done here
                // for this PoC, spacebar unpauses playback
                // we should also maybe send the time to the playback thread so time is adjusted
                ui.input(|i| {
                    if i.key_pressed(egui::Key::Space) {
                        self.tx
                            .enqueue(Packet(0, self.transport, self.stretch_factor))
                            .ok();
                        self.paused = false;
                    }
                });
                self.paused_time += since.mul_f32(self.stretch_factor);
            } else {
                self.transport += since.mul_f32(self.stretch_factor);
                //4 beats per measure
                self.beat = (self.transport.as_micros() as f32 / 1000000.0) * (bpm / 4.0) / 60.0;
                // if note by note is active, check if needs pause
                if self.note_by_note {
                    let start_range = ((self.transport.as_micros() as f32
                        - (since.as_micros() as f32) * self.stretch_factor)
                        / 1000000.0)
                        * (bpm / 4.0)
                        / 60.0;
                    let end_range =
                        (self.transport.as_micros() as f32 / 1000000.0) * (bpm / 4.0) / 60.0;
                    for n in &self.fret_board.notes.0 {
                        //is there a note within the last frame?
                        if (start_range..end_range).contains(&(n.start)) {
                            self.paused = true;
                            self.last_paused = Instant::now();
                            //pause audio thread
                            self.tx
                                .enqueue(Packet(1, self.transport, self.stretch_factor))
                                .ok();
                        }
                    }
                }
            }
            self.fret_board.ui_content(ui, self.beat);
            ctx.request_repaint();
        });
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

#[cfg(test)]
mod test {}
