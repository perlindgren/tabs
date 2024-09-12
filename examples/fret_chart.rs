#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::Stroke;
use std::time::{Duration, Instant};

use egui::*;
use log::*;
use tabs::*;

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

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            fret_board: FretBoard::default(),
            looping: false,
            time_instant: Instant::now(),
            bpm: 20.0,
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
            let beat = (transport.as_micros() as f32 / 1000000.0) * self.bpm / 60.0;

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
    notes: Notes, // perhaps we should use some btree for sorted data structure
}

impl Default for FretBoard {
    fn default() -> Self {
        Self {
            config: Config::default(),
            nr_frets: 6,

            notes: Notes::default(),
        }
    }
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
                    format!("{}", n.string),
                    FontId::monospace(string_space * 0.4),
                    Color32::WHITE,
                );
            } else {
                painter.circle(c, string_space / 2.0, Color32::LIGHT_RED, note_stroke);
                painter.text(
                    c,
                    Align2::CENTER_CENTER,
                    format!("{}", n.string),
                    FontId::monospace(string_space * 0.4),
                    Color32::WHITE,
                );
            }
        }

        // painter.debug_rect(rect, Color32::RED, "here");
        response
    }

    // we assume play head to be displayed one bar in
    // #[inline(always)]
    // pub fn beat_to_pos(&self, play_head: f32, beat: f32) -> f32 {
    //     self.config.beat_pixels * (beat - play_head)
    // }
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
