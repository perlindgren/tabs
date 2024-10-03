use crate::drums::*;

use egui::{Align2, Color32, FontId, Sense, Ui};
use midir::{MidiInput, MidiInputConnection, MidiInputPort};

impl DrumMapping {
    pub fn gui_wizard(&mut self, ui: &mut Ui) -> egui::Response {
        let size = ui.available_size();
        let clip_rect = ui.clip_rect();
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        painter.text(
            clip_rect.center(),
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
    pub config_window: bool,
    pub device: Option<MidiInputPort>,
    pub midi_device_changed: bool,
    pub midi_connection: Option<MidiInputConnection<()>>,
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
            config_window: false,
            device: None,
            midi_device_changed: false,
            midi_connection: None,
        }
    }
}

impl eframe::App for DrumView {
    fn update(&mut self, ctx: &egui::Context, _fram: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Configure Drums").clicked() {
                self.config_window = true;
            }
            egui::Window::new("Drum Wizard")
                .open(&mut self.config_window)
                .collapsible(false)
                .show(ctx, |ui| self.drum_mapping.gui_wizard(ui));
            // MIDI Device selection
            let midi_input = MidiInput::new("MIDI").unwrap();

            egui::ComboBox::from_label("MIDI Input Device")
                .selected_text(if let Some(device) = &self.device {
                    midi_input.port_name(&device).unwrap()
                } else {
                    "Select Input".to_string()
                })
                .show_ui(ui, |ui| {
                    for port in midi_input.ports() {
                        let port_name = midi_input.port_name(&port).unwrap();
                        let dropdown_item =
                            ui.selectable_value(&mut self.device, Some(port), port_name);

                        if dropdown_item.clicked() {
                            // A MIDI Device has been selected, reconnect
                            if self.device.is_some() {
                                println!("clicked");
                                self.midi_device_changed = true;
                            }
                        };
                    }
                });

            if let Some(device) = &self.device.as_ref() {
                let name = midi_input.port_name(&device);
                if self.midi_device_changed {
                    println!("connecting");
                    self.midi_connection = Some(
                        midi_input
                            .connect(
                                &device,
                                &name.unwrap(),
                                move |stamp, message, _| {
                                    println!("{}: {:?} (len = {})", stamp, message, message.len());
                                },
                                (),
                            )
                            .unwrap(),
                    );
                    self.midi_device_changed = false;
                }
            }
        });
    }
}
