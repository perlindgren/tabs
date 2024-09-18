#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::Stroke;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use clap::Parser;
use egui::*;
use log::*;
use scorelib::gp;
use std::{fs, io::Read, path::Path};
use tabs::*;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short = 'p', long, help = "Input file path")]
    path: String,
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
    fret_board: FretBoard,
    looping: bool,
    time_instant: Instant,
    bpm: f32,
    start_instant: Instant,
}

impl<'a> MyApp {
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
        println!("Operned song {} by {}", song.name, song.artist);

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
        let tuning: Rc<RefCell<dyn Tuning>> = if strings.as_slice() == eadgbe.tuning() {
            Rc::new(RefCell::new(eadgbe))
        } else if strings.as_slice() == eadg.tuning() {
            Rc::new(RefCell::new(eadg))
        } else {
            panic!("Unsupported tuning")
        };
        println!("Tuning: {:?}", tuning.borrow().tuning());
        let mut fretnotes = vec![];
        //insert two measures of silence
        let mut current_time = 2.0;
        /*let headers = song.measure_headers;
        let header_index = measure_1.header_index;
        let header = headers.get(header_index).unwrap();
        let tempo = header.tempo;*/
        let tempo = song.tempo;
        println!("tempo {}", tempo);
        for measure in &track.measures {
            let voice = measure.voices.get(0).unwrap();
            for beat in &voice.beats {
                for note in &beat.notes {
                    let fretnote = FretNote::new(
                        (note.string - 1) as u8, //zero indexed...
                        note.value as u8,
                        current_time as f32,
                        None,
                        tuning.clone(),
                    );
                    fretnotes.push(fretnote);
                    current_time += 1.0 / beat.duration.value as f64;
                    println!("Duration: {}", beat.duration.value as f64);
                }
            }
        }
        /*for n in fretnotes.clone() {
            println!(
                "String:{}, Fret: {}, Start time: {}",
                n.string, n.fret, n.start
            );
        }*/
        let fretnotes = FretNotes(fretnotes);
        Self {
            fret_board: FretBoard::new(fretnotes),
            looping: false,
            time_instant: Instant::now(),
            bpm: tempo as f32,
            start_instant: Instant::now(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let now = Instant::now();
            let since = now - self.time_instant;
            let one_sec = Duration::from_secs(1);
            let transport = now - self.start_instant;
            //4 beats per measure
            let beat = (transport.as_micros() as f32 / 1000000.0) * (self.bpm / 4.0) / 60.0;

            self.time_instant = now;

            let f = (one_sec.as_micros() / since.as_micros()) as u32;

            ui.label(format!("Freq: {:?}", f));

            // if f < 59 || f > 61 {
            //     debug!("frame-rate {}", f);
            // }

            ui.label(format!("Transport: {:?}", transport));
            ui.label(format!("Beat {}, Pos {}", 1 + beat as u32 % 4, beat as u32));

            if ui.checkbox(&mut self.looping, "looping").clicked()
            // || ui.checkbox(&mut self.warping, "warping").clicked()
            {
                trace!("something clicked, clip_rect {:?}", ui.clip_rect());
            }
            if ui.button("restart").clicked() {
                trace!("restart {:?}", ui.clip_rect());
                self.start_instant = Instant::now();
            }
            self.fret_board.ui_content(ui, beat);
            ctx.request_repaint();
        });
    }
}

struct FretBoard {
    config: Config,
    nr_frets: u8,
    notes: FretNotes, // perhaps we should use some btree for sorted data structure
}

#[derive(Debug)]
struct Config {
    beats: f32,
    subs: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            beats: 4.0,
            subs: 4.0,
        }
    }
}

impl FretBoard {
    pub fn new(notes: FretNotes) -> Self {
        Self {
            config: Config::default(),
            nr_frets: 6,
            notes,
        }
    }
    pub fn ui_content(&mut self, ui: &mut Ui, play_head: f32) -> egui::Response {
        let size = ui.available_size();
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        let rect = response.rect;
        trace!("rect {:?}", rect);

        let string_space = rect.height() / (self.nr_frets as f32);

        let fret_stroke = Stroke::new(1.0, Color32::from_gray(128));
        // draw strings
        for i in 0..self.nr_frets {
            let y = string_space * (0.5 + i as f32) + rect.top();
            trace!("i {}, y {}", i, y);
            painter.line_segment(
                [(rect.left(), y).into(), (rect.right(), y).into()],
                fret_stroke,
            );
        }

        // draw bars,
        let bar_stroke = Stroke::new(1.0, Color32::from_gray(255));
        let sub_stroke = Stroke::new(1.0, Color32::from_gray(64));

        let subs = self.config.beats * self.config.subs;
        let bar_pixels = rect.width() / self.config.beats;
        let sub_pixels = bar_pixels / self.config.subs;

        for i in 0..subs as usize {
            let x = sub_pixels * i as f32 - play_head * bar_pixels;
            let x = x % rect.width();
            let x = if x < 0.0 { x + rect.width() } else { x };
            let x = x + rect.left();
            let x = x.round();

            painter.line_segment(
                [(x, rect.top()).into(), (x, rect.bottom()).into()],
                if i % self.config.subs as usize == 0 {
                    bar_stroke
                } else {
                    sub_stroke
                },
            );
            painter.text(
                (x, 20.0 + rect.top()).into(),
                Align2::CENTER_CENTER,
                if false {
                    format!(
                        "{}/{}",
                        play_head.trunc() as usize + i,
                        i % self.config.subs as usize
                    )
                } else {
                    format!("{}", i % self.config.subs as usize)
                },
                FontId::monospace(string_space * 0.4),
                Color32::WHITE,
            );
        }

        // draw note
        let note_stroke = Stroke::new(2.0, Color32::WHITE);

        for n in &self.notes.0 {
            let y = string_space * (0.5 + n.string as f32) + rect.top();
            let c = (rect.left() + (n.start - play_head) * bar_pixels, y).into();

            if n.start > play_head + self.config.beats || n.start < play_head {
                trace!("skipping {}", n.start);
            }
            if let Some(ext) = n.ext {
                let top = string_space * (n.string as f32) + rect.top();
                let bottom = string_space * (1.0 + n.string as f32) + rect.top();
                let left = rect.left() + (n.start - play_head) * bar_pixels - string_space * 0.5;
                let right = rect.left() + (ext - play_head) * bar_pixels + string_space * 0.5;

                painter.rect(
                    [(left, top).into(), (right, bottom).into()].into(),
                    string_space * 0.1,
                    Color32::LIGHT_RED,
                    note_stroke,
                );
                painter.text(
                    c,
                    Align2::CENTER_CENTER,
                    format!("{}", n.fret),
                    FontId::monospace(string_space * 0.4),
                    Color32::WHITE,
                );
            } else {
                painter.circle(c, string_space / 4.0, Color32::LIGHT_RED, note_stroke);
                painter.text(
                    c,
                    Align2::CENTER_CENTER,
                    format!("{}", n.fret),
                    FontId::monospace(string_space * 0.4),
                    Color32::WHITE,
                );
            }
        }

        response
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
mod test {
    // use crate::FretBoard;

    // #[test]
    // fn test_beat_to_pos() {
    //     let fb = FretBoard::default();

    //     let r = fb.beat_to_pos(2.0, 1.0);
    //     println!("r {}", r);
    // }

    #[test]
    fn fmod() {
        let range = 2.0;
        for i in 0..20 {
            println!("{}", range + (i as f32 * 1.05) % range);
        }
    }
}
