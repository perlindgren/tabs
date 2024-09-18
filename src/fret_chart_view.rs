use crate::FretNotes;
use egui::*;
use log::*;

#[derive(Debug)]
pub struct FretChart {
    config: Config,
    nr_frets: u8,
    notes: FretNotes, // perhaps we should use some btree for sorted data structure
}

impl Default for FretChart {
    fn default() -> Self {
        Self {
            config: Config::default(),
            nr_frets: 6,

            notes: FretNotes(vec![]),
        }
    }
}

#[derive(Debug)]
pub struct Config {
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

impl FretChart {
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
