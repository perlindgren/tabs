#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use clap::Parser;
use cpal::{traits::*, *};
use eframe::egui;
use egui::{Align2, Color32, FontId, Sense};
use heapless::spsc::*;
use num::complex::ComplexFloat;
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

struct Packet(u8, Duration, f32);

const QUEUE_SIZE: usize = 8;
type Q = Queue<Packet, QUEUE_SIZE>;

type P = Producer<'static, Packet, QUEUE_SIZE>;

const AUDIO_QUEUE_SIZE: usize = 1024; // in f32
type AQ = Queue<f32, { AUDIO_QUEUE_SIZE * 2 }>;
type AC = Consumer<'static, f32, { AUDIO_QUEUE_SIZE * 2 }>;

const FS: usize = 2048 * 2;
const DETECTION_THRESHOLD: f32 = 0.00001; // likeness factor between filter and signal
                                          // will probably be affected by signal volume and instrument tuning and
                                          // intonation so should be derived from the actual signal eventually.

//in millis
const HIT_WINDOW: u128 = 300;
use log::*;
use scorelib::{enums::NoteType, gp};
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
    let spsc: &'static mut AQ = {
        static mut SPSC: AQ = Queue::new();
        #[allow(static_mut_refs)]
        unsafe {
            &mut SPSC
        }
    };

    let (mut producer, consumer) = spsc.split();
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .expect("no input device available");
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: SampleRate(FS as u32),
        buffer_size: BufferSize::Fixed(64 * 4), // 64 samples
    };
    let input_stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _info: &cpal::InputCallbackInfo| {
                //println!("INPUT");
                for &sample in data {
                    if producer.enqueue(sample).is_err() {
                        //println!("spsc queue full");
                    }
                }
            },
            move |err| {
                // react to errors here.
                println!("stream error {:?}", err)
            },
            None, // None=blocking, Some(Duration)=timeout
        )
        .expect("failed to configure input stream");
    input_stream.play().unwrap();

    eframe::run_native(
        "Fret Test",
        options,
        Box::new(move |cc| {
            let app = MyApp::new(cc, consumer);
            Ok(Box::new(app))
        }),
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
    audio_consumer: AC,
    in_data: [f32; FS],
    ptr: usize,
    expected_notes: Vec<(FretNote, usize)>,
    finished: bool,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>, consumer: AC) -> Self {
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

        let strings: Vec<Note> = strings.iter().map(|item| (*item).into()).collect();

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
        let mut current_time = 2.0;
        //let mut current_time = 2.0;
        let tempo = song.tempo;
        println!("tempo {}", tempo);
        for measure in &track.measures.as_slice()[0..10] {
            let voice = measure.voices.first().unwrap();
            for beat in &voice.beats {
                for note in &beat.notes {
                    if note.kind == NoteType::Normal {
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
        }
        //loop {}
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
            //playback.try_seek(Duration::from_secs_f32(0.7)).ok();
            playback.set_volume(0.5);
            let mut playhead = playback.get_pos();
            loop {
                if playhead != playback.get_pos() {
                    playhead = playback.get_pos();
                }
                println!("{}", playhead.as_secs_f32());

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
            audio_consumer: consumer,
            in_data: [0.0; FS],
            ptr: 0,
            expected_notes: vec![],
            finished: false,
        }
    }
}
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let now = Instant::now();
            while let Some(s) = self.audio_consumer.dequeue() {
                self.in_data[self.ptr] = s; // most recent sample
                self.ptr = (self.ptr + 1) % FS; // next
            }
            let since = now - self.time_instant;
            let one_sec = Duration::from_secs(1);
            //now - (self.start_instant + self.paused_time);
            let bpm = self.bpm;
            if self.beat > 2.0 {
                if !self.playing_audio {
                    //start playback
                    self.tx
                        .enqueue(Packet(0, self.transport, self.stretch_factor))
                        .ok();
                    self.playing_audio = true;
                }
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
            if !self.finished {
                if self.paused {
                    let harmonic: f32 = 4.0;
                    let p: u8 = 40;

                    let mut remove_indices = vec![];
                    for (i, expected) in self.expected_notes.iter().enumerate() {
                        let note: Note = expected.0.clone().into();
                        let expected_fr: Hz = note.into();
                        //println!("expected_fr: {}", expected_fr.0);
                        let expected_fr: f32 = expected_fr.0 * harmonic;
                        let sin_cos_hann = tabs::dsp::sin_cos_hann(FS, expected_fr, p);
                        let c = tabs::dsp::conv_at_k(&self.in_data, &sin_cos_hann, FS / 2);
                        if c.abs() > DETECTION_THRESHOLD {
                            remove_indices.push(i);

                            //  println!("CORRECT NOTE DETECTED: c: {}", c.abs());
                        }
                    }
                    remove_indices.reverse();
                    // reverse to preserve expected ordering when removing
                    for i in remove_indices {
                        let idx = self.expected_notes.get(i).unwrap().1;
                        let note_ref = self.fret_board.notes.0.get_mut(idx).unwrap();
                        note_ref.hit = true;
                        self.expected_notes.remove(i);
                    }
                    // if we are not awaiting any more notes, unpause playback
                    if self.expected_notes.is_empty() {
                        self.tx
                            .enqueue(Packet(0, self.transport, self.stretch_factor))
                            .ok();
                        self.paused = false;
                    }
                    self.paused_time += since.mul_f32(self.stretch_factor);
                } else {
                    self.transport += since.mul_f32(self.stretch_factor);
                    //4 beats per measure
                    self.beat =
                        (self.transport.as_micros() as f32 / 1000000.0) * (bpm / 4.0) / 60.0;
                    // if note by note is active, check if needs pause
                    if self.note_by_note {
                        let start_range = ((self.transport.as_micros() as f32
                            - since.as_micros() as f32)
                            / 1000000.0)
                            * (self.bpm / 4.0)
                            / 60.0;
                        let end_range = (self.transport.as_micros() as f32 / 1000000.0)
                            * (self.bpm / 4.0)
                            / 60.0;

                        let mut i = 0;
                        let mut finished = true;
                        for n in self.fret_board.notes.0.iter() {
                            //is there a note within the last frame?
                            if n.start > end_range {
                                finished = false;
                            }
                            if (start_range..end_range).contains(&(n.start)) {
                                self.paused = true;
                                self.expected_notes.push((n.clone(), i));
                                self.last_paused = Instant::now();
                                //pause audio thread
                                self.tx
                                    .enqueue(Packet(1, self.transport, self.stretch_factor))
                                    .ok();
                            }

                            i += 1;
                        }
                        if finished {
                            self.finished = true
                        };
                    } else {
                        let start_window = (self.transport.as_millis() as f32 / 1000.0
                            - ((HIT_WINDOW as f32 / 2000.0) as f32))
                            * (self.bpm / 4.0)
                            / 60.0;
                        let end_window = ((self.transport.as_millis() as f32 / 1000.0)
                            + (HIT_WINDOW as f32 / 2000.0))
                            * (self.bpm / 4.0)
                            / 60.0;
                        //println!("{}:{}", start_window, end_window);
                        let mut finished = true;
                        for n in self.fret_board.notes.0.iter_mut() {
                            if (start_window..end_window).contains(&n.start) && !n.hit {
                                let harmonic = 4.0;
                                let p = 40;
                                let note: Note = n.into();
                                let expected_fr: Hz = note.into();
                                //      println!("expected_fr: {}", expected_fr.0);
                                let expected_fr: f32 = expected_fr.0 * harmonic;
                                let sin_cos_hann = tabs::dsp::sin_cos_hann(FS, expected_fr, p);
                                let c = tabs::dsp::conv_at_k(&self.in_data, &sin_cos_hann, FS / 2);
                                if c.abs() > DETECTION_THRESHOLD {
                                    n.hit = true;
                                    //        println!("CORRECT NOTE DETECTED: c: {}", c.abs());
                                }
                            }
                            if n.start > start_window {
                                finished = false;
                            }
                        }
                        if finished {
                            self.finished = true
                        };
                    }
                }
                self.fret_board.ui_content(ui, self.beat);
            } else {
                let size = ui.available_size();
                let (response, painter) = ui.allocate_painter(size, Sense::hover());
                let rect = response.rect;
                let mut correct_notes = 0;
                for n in &self.fret_board.notes.0 {
                    if n.hit {
                        correct_notes += 1;
                    }
                }

                let accuracy =
                    (correct_notes as f32 / self.fret_board.notes.0.len() as f32) * 100.0;
                painter.text(
                    (rect.width() / 2.0, rect.height() / 2.0).into(),
                    Align2::CENTER_CENTER,
                    format!("SONG FINISHED\nACCURACY: {}%\nYOU WIN?", accuracy),
                    FontId::monospace(50.0),
                    Color32::WHITE,
                );
            }
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
