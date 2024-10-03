use crate::drums::*;
use egui::{Align2, Color32, FontId, Response, Sense, Stroke, Ui};
use heapless::spsc::*;
use midir::{MidiInput, MidiInputConnection, MidiInputPort};

type Packet = u8;
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
            if !self.mapping.0.contains_key(&(msg as i16)) {
                self.mapping.0.insert(msg as i16, self.state);
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
            if let Some(rx) = &mut self.consumer {
                if let Some(msg) = rx.dequeue() {
                    if !self.drum_mapping_gui.is_opened {
                        if let Some(drum) = self.drum_mapping_gui.mapping.0.get(&(msg as i16)) {
                            for d in self.drums.iter_mut() {
                                if &d.0 == drum {
                                    d.1 = 10; // change color for 10 frames to indicate hit
                                }
                                if drum == &Drum::Kick {
                                    self.kick = 10;
                                }
                            }
                        }
                        //self.counter += 1;
                        //println!("main GUI msg:{} {}", msg, self.counter);
                    } else {
                        self.producer.enqueue(msg).ok();
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
                    for port in midi_input.ports() {
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

            if let Some(device) = &self.device.as_ref() {
                let name = midi_input.port_name(&device);
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

                    self.midi_connection = Some(
                        midi_input
                            .connect(
                                &device,
                                &name.unwrap(),
                                move |_, message, _| {
                                    tx.enqueue(message[1]).ok();
                                },
                                (),
                            )
                            .unwrap(),
                    );
                    self.midi_device_changed = false;
                }
            }

            // wizard
            if ui.button("Configure Drums").clicked() {
                self.drum_mapping_gui.is_opened = true;
            }
            let mut is_opened = self.drum_mapping_gui.is_opened;
            egui::Window::new("Drum Wizard")
                .open(&mut is_opened)
                .collapsible(false)
                .show(ctx, |ui| self.drum_mapping_gui.gui_wizard(ui));

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

            // kick
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
