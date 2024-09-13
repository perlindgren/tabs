#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::Stroke;
use std::{
    // marker::PhantomData,
    time::{Duration, Instant},
};

use egui::*;
use log::*;
use tabs::*;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    debug!("env_logger started");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1920.0, 540.0]),
        // vsync: false,
        ..Default::default()
    };

    eframe::run_native(
        "Fret Test",
        options,
        Box::new(|cc| {
            let app: MyApp<6, EADGBE> = MyApp::new(cc);
            Ok(Box::new(app))
        }),
    )
}

struct MyApp<const N: usize, T>
where
    T: Tuning<N>,
{
    fret_board: FretBoard<N, T>,
    looping: bool,
    time_instant: Instant,
    bpm: f32,
    start_instant: Instant,
}

impl MyApp<6, EADGBE> {
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

impl<const N: usize, T> eframe::App for MyApp<N, T>
where
    T: Tuning<N>,
{
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

            if f < 59 || f > 61 {
                debug!("frame-rate {}", f);
            }

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

struct FretBoard<const N: usize, T>
where
    T: Tuning<N>,
{
    config: Config,
    nr_frets: u8,
    notes: Notes<N, T>, // perhaps we should use some btree for sorted data structure
                        // _marker: PhantomData<T>,
}

impl Default for FretBoard<6, EADGBE> {
    fn default() -> Self {
        Self {
            config: Config::default(),
            nr_frets: 6,
            // notes: Notes(vec![]), // Notes<N, T>::default(),
            notes: Notes::default(),
        }
    }
}

#[derive(Debug)]
struct Config {
    frets: Vec<(f32, f32)>, // mid fret, fret pos
    sum_frets: f32,         // last fret position
}
impl Config {
    fn new(nr_frets: usize) -> Self {
        const FACTOR: f32 = 17.817154;

        let mut sum_frets = 0.0;
        let mut frets = vec![];

        let mut scale_length = 1.0;

        for _ in 0..nr_frets {
            let next = scale_length / FACTOR;

            frets.push((next / 2.0 + sum_frets, next + sum_frets));

            scale_length -= next;
            sum_frets += next;
        }

        Config { frets, sum_frets }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new(22)
    }
}

impl<const N: usize, T> FretBoard<N, T>
where
    T: Tuning<N>,
{
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

        // draw frets,
        let fret_stroke = Stroke::new(1.0, Color32::from_gray(255));
        let scaling = rect.width() / self.config.sum_frets;

        self.config.frets.iter().enumerate().for_each(|(i, fret)| {
            painter.vline(
                rect.left() + fret.1 * scaling,
                Rangef::new(rect.top(), rect.bottom()),
                fret_stroke,
            );
            painter.text(
                (
                    rect.left() + fret.0 * scaling,
                    rect.width() * 0.005 + rect.top(),
                )
                    .into(),
                Align2::CENTER_CENTER,
                format!("{}", i + 1),
                FontId::monospace(rect.width() * 0.01),
                Color32::WHITE,
            );
        });

        // draw note
        let note_stroke = Stroke::new(2.0, Color32::WHITE);

        for n in &self.notes.0 {
            //
            if n.fret > 0 {
                if let Some(fret) = self.config.frets.get(n.fret as usize - 1) {
                    // debug!("note n {:?}, fret {:?}", n, fret);
                    let y = string_space * (0.5 + n.string as f32) + rect.top();

                    painter.circle(
                        (rect.left() + fret.0 * scaling, y).into(),
                        string_space / 2.0,
                        Color32::LIGHT_RED,
                        note_stroke,
                    );
                }
            }

            //     let y = string_space * (0.5 + n.fret as f32) + rect.top();
            //     let c = (rect.left() + (n.on - play_head) * bar_pixels, y).into();

            //     if n.on > play_head + self.config.beats || n.on < play_head {
            //         trace!("skipping {}", n.on);
            //     }
            //     if let Some(ext) = n.ext {
            //         let top = string_space * (n.fret as f32) + rect.top();
            //         let bottom = string_space * (1.0 + n.fret as f32) + rect.top();
            //         let left = rect.left() + (n.on - play_head) * bar_pixels - string_space * 0.5;
            //         let right = rect.left() + (ext - play_head) * bar_pixels + string_space * 0.5;

            //         painter.rect(
            //             [(left, top).into(), (right, bottom).into()].into(),
            //             string_space * 0.1,
            //             Color32::LIGHT_RED,
            //             note_stroke,
            //         );
            //         painter.text(
            //             c,
            //             Align2::CENTER_CENTER,
            //             format!("{}", n.pos),
            //             FontId::monospace(string_space * 0.4),
            //             Color32::WHITE,
            //         );
            //     } else {
            //         painter.circle(c, string_space / 2.0, Color32::LIGHT_RED, note_stroke);
            //         painter.text(
            //             c,
            //             Align2::CENTER_CENTER,
            //             format!("{}", n.pos),
            //             FontId::monospace(string_space * 0.4),
            //             Color32::WHITE,
            //         );
            //     }
        }

        // painter.debug_rect(rect, Color32::RED, "here");
        response
    }
}

#[cfg(test)]
mod test {

    use crate::Config;

    #[test]
    fn fmod() {
        let range = 2.0;
        for i in 0..20 {
            println!("{}", range + (i as f32 * 1.05) % range);
        }
    }

    #[test]
    fn test_config() {
        let config = Config::default();
        println!("config {:?}", config);
    }
}
