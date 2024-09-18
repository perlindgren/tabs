#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use std::time::{Duration, Instant};

use log::*;
use tabs::fret_chart_view::*;

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
    fret_chart: FretChart,
    looping: bool,
    time_instant: Instant,
    bpm: f32,
    start_instant: Instant,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            fret_chart: FretChart::default(),
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
            self.fret_chart.ui_content(ui, beat);
            ctx.request_repaint();
        });
    }
}

#[cfg(test)]
mod test {}
