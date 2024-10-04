use crate::drums::*;
use egui::{Align2, Color32, FontId, Sense, Stroke, Ui};
use heapless::spsc::*;
use midir::{MidiInput, MidiInputConnection, MidiInputPort};

type Packet = (u8, u8); // ID, Velocity
const QUEUE_SIZE: usize = 8; //2 feet + 2 hands * 2 should be enough
type Q = Queue<Packet, QUEUE_SIZE>;

type P = Producer<'static, Packet, QUEUE_SIZE>;

pub type C = Consumer<'static, Packet, QUEUE_SIZE>;

pub struct DrumMappingGui {
    pub mapping: DrumMapping,
    pub consumer: C,
    pub state: Drum,     // which drum are we currently configuring?
    pub hit_counter: u8, // how many times have we received a message for this particular drum
    pub is_opened: bool,
}

impl DrumMappingGui {
    pub fn new(mapping: DrumMapping, consumer: C) -> Self {
        DrumMappingGui {
            mapping,
            consumer,
            state: Drum::Snare,
            hit_counter: 0,
            is_opened: false,
        }
    }
    pub fn gui_wizard(&mut self, ui: &mut Ui) -> egui::Response {
        let size = ui.available_size();
        let clip_rect = ui.clip_rect();
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        let text = format!("Hit {:?}", self.state);
        if let Some(msg) = self.consumer.dequeue() {
            self.hit_counter += 1;
            if !self.mapping.0.contains_key(&(msg.0 as i16)) {
                self.mapping.0.insert(msg.0 as i16, self.state);
            }
            if self.hit_counter == 10 {
                self.state = match self.state {
                    Drum::Snare => Drum::HihatClosed,
                    Drum::HihatClosed => Drum::HihatOpen,
                    Drum::HihatOpen => Drum::Crash,
                    Drum::Crash => Drum::Tom1,
                    Drum::Tom1 => Drum::Tom2,
                    Drum::Tom2 => Drum::Ride,
                    Drum::Ride => Drum::FloorTom,
                    Drum::FloorTom => Drum::Kick,
                    Drum::Kick => {
                        println!("done");
                        self.is_opened = false;
                        Drum::Snare
                    }
                };
                self.hit_counter = 0;
            }
        };
        painter.text(
            clip_rect.center(),
            Align2::CENTER_CENTER,
            text,
            FontId::monospace(50.0),
            Color32::RED,
        );
        response
    }
}

//#[derive(Debug)]
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
    pub drum_mapping_gui: DrumMappingGui,
    //pub config_window: bool,
    pub device: Option<MidiInputPort>,
    pub midi_device_changed: bool,
    pub midi_connection: Option<MidiInputConnection<()>>,
    pub consumer: Option<C>,
    pub counter: usize,
    pub producer: P,
    pub drums: [(Drum, u8); 7],
    pub kick: u8,
}

impl DrumView {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        chart: DrumChart,
        drum_mapping: DrumMapping,
    ) -> Self {
        let spsc: &'static mut Q = {
            static mut SPSC: Q = Queue::new();
            #[allow(static_mut_refs)]
            unsafe {
                &mut SPSC
            }
        };
        let (tx, rx) = spsc.split();

        let drum_mapping_gui = DrumMappingGui::new(drum_mapping, rx);

        let drums = [
            (Drum::Snare, 0),
            (Drum::HihatClosed, 0),
            (Drum::Crash, 0),
            (Drum::Tom1, 0),
            (Drum::Tom2, 0),
            (Drum::Ride, 0),
            (Drum::FloorTom, 0),
        ];
        DrumView {
            chart,
            drum_mapping_gui,
            //config_window: false,
            device: None,
            midi_device_changed: false,
            midi_connection: None,
            consumer: None,
            counter: 0, //remove this later
            producer: tx,
            drums,
            kick: 0,
        }
    }
}

impl eframe::App for DrumView {
    fn update(&mut self, ctx: &egui::Context, _fram: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // handle potential MIDI messages

            if let Some(rx) = &mut self.consumer {
                if let Some(msg) = rx.dequeue() {
                    // If note velocity is non-zero
                    if msg.1 != 0 {
                        if !self.drum_mapping_gui.is_opened {
                            if let Some(drum) = self.drum_mapping_gui.mapping.0.get(&(msg.0 as i16))
                            {
                                // set the hit drum as hit, the value here is used as a frame
                                // counter, decreasing by 1 each frame
                                for d in self.drums.iter_mut() {
                                    if &d.0 == drum {
                                        d.1 = 10; // change color for 10 frames to indicate hit
                                    }
                                    if drum == &Drum::Kick {
                                        self.kick = 10;
                                    }
                                }
                            }
                        } else {
                            // If Config Wizard is open, forward the MIDI message to it.
                            self.producer.enqueue(msg).ok();
                        }
                    }
                }
            }

            // MIDI Device selection
            let midi_input = MidiInput::new("MIDI").unwrap();

            egui::ComboBox::from_label("MIDI Input Device")
                .selected_text(if let Some(device) = &self.device {
                    midi_input.port_name(&device).unwrap()
                } else {
                    "Select Input".to_string()
                })
                .show_ui(ui, |ui| {
                    // Create a dropdown option for each detected port
                    for port in midi_input.ports() {
                        // if this unwrap fails, we should probably panic in any case
                        let port_name = midi_input.port_name(&port).unwrap();
                        let dropdown_item =
                            ui.selectable_value(&mut self.device, Some(port), port_name);
                        if dropdown_item.clicked() {
                            // A MIDI Device has been selected, reconnect
                            if self.device.is_some() {
                                self.midi_device_changed = true;
                            }
                        };
                    }
                });

            // Listen to the selected MIDI Device
            if let Some(device) = &self.device.as_ref() {
                let name = midi_input.port_name(&device);
                // If the device has changed, create a new SPSC
                if self.midi_device_changed {
                    let spsc: &'static mut Q = {
                        static mut SPSC: Q = Queue::new();
                        #[allow(static_mut_refs)]
                        unsafe {
                            &mut SPSC
                        }
                    };

                    let (mut tx, rx) = spsc.split();

                    self.consumer = Some(rx);
                    // The producer is passed to the MIDI callback closure, enabling communication
                    // with the GUI
                    self.midi_connection = Some(
                        midi_input
                            .connect(
                                &device,
                                &name.unwrap(),
                                move |_, message, _| {
                                    // This is MIDI standard, 2nd byte of message is the MIDI code
                                    // 3rd is the note velocity
                                    tx.enqueue((message[1], message[2])).ok();
                                },
                                (),
                            )
                            .unwrap(),
                    );
                    // We've now handled the device change
                    self.midi_device_changed = false;
                }
            }

            // Spawn Config Wizard
            if ui.button("Configure Drums").clicked() {
                self.drum_mapping_gui.is_opened = true;
            }
            let mut is_opened = self.drum_mapping_gui.is_opened;
            egui::Window::new("Drum Wizard")
                .open(&mut is_opened)
                .collapsible(false)
                .show(ctx, |ui| self.drum_mapping_gui.gui_wizard(ui));

            if ui.button("Load Drum Mapping").clicked() {
                let path_option = rfd::FileDialog::new().pick_file();
                if let Some(path) = path_option {
                    if let Ok(data) = std::fs::read_to_string(path) {
                        let drum_mapping: Result<DrumMapping, serde_json::Error> =
                            serde_json::from_str(&data);
                        if let Ok(drum_mapping) = drum_mapping {
                            self.drum_mapping_gui.mapping = drum_mapping;
                        }
                    }
                }
            }

            if ui.button("Save Drum Mapping").clicked() {
                let path_option = rfd::FileDialog::new().save_file();
                if let Some(path) = path_option {
                    let serialized_drum_mapping =
                        serde_json::to_string(&self.drum_mapping_gui.mapping).unwrap();
                    std::fs::write(path, serialized_drum_mapping).ok();
                }
            }

            // draw chart board
            let size = ui.available_size();
            let (response, painter) = ui.allocate_painter(size, Sense::hover());
            let rect = response.rect;
            // lane bars
            let len = self.drums.len();
            let lane_stroke = Stroke::new(2.0, Color32::from_gray(255));
            for i in 1..=len {
                let x = (rect.width() / len as f32) * i as f32;
                painter.line_segment(
                    [(x, rect.top()).into(), (x, rect.bottom()).into()],
                    lane_stroke,
                );
            }

            // drums
            let drum_stroke = Stroke::new(2.0, Color32::WHITE);
            for (i, drum) in self.drums.iter_mut().enumerate() {
                // drum.1 is a frame counter as described in the MIDI SPSC handler
                // we want to light up for a *couple* of frames when hit, for now it's 10.
                let fill_color = if drum.1 > 0 {
                    drum.1 -= 1;
                    Color32::LIGHT_GREEN
                } else {
                    Color32::LIGHT_RED
                };
                let x = (rect.width() / len as f32) * i as f32 + rect.width() / (len as f32 * 2.0);
                let radius = (rect.width() / len as f32) / 2.0;
                painter.circle(
                    (x, rect.height() - radius).into(),
                    radius,
                    fill_color,
                    drum_stroke,
                );
            }

            // kick bar
            let kick_stroke = Stroke::new(5.0, Color32::LIGHT_BLUE);
            if self.kick > 0 {
                self.kick -= 1;
                let radius = (rect.width() / len as f32) / 2.0;
                painter.line_segment(
                    [
                        (0.0, rect.height() - radius).into(),
                        (rect.width(), rect.height() - radius).into(),
                    ],
                    kick_stroke,
                );
            }

            //let drum_stroke = Stroke::new(2.0, Color32::WHITE);
        });
        ctx.request_repaint();
    }
}
