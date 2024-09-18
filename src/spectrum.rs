use spectrum_analyzer::FrequencySpectrum;

use egui::*;
use log::*;

#[derive(Default)]
pub struct SpectrumView {}

impl SpectrumView {
    pub fn ui_content(&mut self, ui: &mut Ui, spectrums: Vec<FrequencySpectrum>) -> egui::Response {
        // ui.label(format!("max {:?}", self.max));

        let size = ui.available_size();
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        let rect = response.rect;
        trace!("rect {:?}", rect);

        let fft_strokes = [
            Stroke::new(1.0, Color32::WHITE),
            Stroke::new(1.0, Color32::YELLOW),
            Stroke::new(1.0, Color32::GREEN),
            Stroke::new(1.0, Color32::BLUE),
        ];

        // for (f, v) in spectrum.data().iter().take(20) {
        //     print!("{}, ", f.val())
        // }
        // println!();

        for (i, s) in spectrums.iter().rev().enumerate() {
            // draw spectrum
            for (f, v) in s.data().iter() {
                let x: f32 = f.val();
                let v: f32 = v.val() * 50.0;
                painter.vline(
                    x + rect.left(),
                    Rangef::new(rect.top() + rect.height() * (1.0 - v), rect.bottom()),
                    fft_strokes[i],
                );
            }
        }

        // painter.vline(
        //     82.0 + rect.left(), // our E bin
        //     Rangef::new(rect.top(), rect.height() / 2.0),
        //     fft_stroke,
        // );

        // painter.debug_rect(rect, Color32::RED, "here");
        response
    }
}
