use crate::drums::*;

use egui::{Align2, Color32, FontId, Sense, Ui};

impl DrumMapping {
    pub fn gui_wizard(&mut self, ui: &mut Ui) -> egui::Response {
        let size = ui.available_size();
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let rect = response.rect;
        painter.text(
            (rect.width() / 2.0, rect.height() / 2.0).into(),
            Align2::CENTER_CENTER,
            "WIZARD",
            FontId::monospace(50.0),
            Color32::WHITE,
        );
        response
    }
}

#[derive(Debug)]
pub struct DrumChart {
    pub notes: DrumNotes,
}

impl Default for DrumChart {
    fn default() -> Self {
        Self {
            notes: DrumNotes {
                notes: vec![],
                tempo: 0,
            },
        }
    }
}

impl DrumChart {
    pub fn new(notes: DrumNotes) -> Self {
        DrumChart { notes }
    }

    pub fn gui(&mut self, ui: &mut Ui) -> egui::Response {
        let size = ui.available_size();
        let (response, _painter) = ui.allocate_painter(size, Sense::hover());

        //let rect = response.rect;
        response
    }
}

pub struct DrumView {
    pub chart: DrumChart,
    pub drum_mapping: DrumMapping,
}

impl DrumView {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        chart: DrumChart,
        drum_mapping: DrumMapping,
    ) -> Self {
        DrumView {
            chart,
            drum_mapping,
        }
    }
}

impl eframe::App for DrumView {
    fn update(&mut self, ctx: &egui::Context, _fram: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Configure Drums").clicked() {
                egui::Window::new("Drum Wizard").show(ctx, |ui| {
                    self.drum_mapping.gui_wizard(ui);
                });
            }
        });
    }
}
