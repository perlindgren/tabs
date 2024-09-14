use cpal::{traits::*, *};
use egui_plot::HLine;
use heapless::spsc::*;

use egui::*;
use log::*;

const QUEUE_SIZE: usize = 1024; // in f32
type Q = Queue<f32, { QUEUE_SIZE * 2 }>;
type C = Consumer<'static, f32, { QUEUE_SIZE * 2 }>;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    debug!("env_logger started");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1920.0, 540.0]),
        // vsync: false,
        ..Default::default()
    };

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("no input device available");

    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: SampleRate(48000),
        buffer_size: BufferSize::Fixed(256 as u32), // in bytes
    };

    let spsc: &'static mut Q = {
        static mut SPSC: Q = Queue::new();
        unsafe { &mut SPSC }
    };

    let (mut producer, consumer) = spsc.split();

    let input_stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], info: &cpal::InputCallbackInfo| {
                // react to stream events and read or write stream data here.
                println!("data len {}, info {:?}", data.len(), info);

                for &sample in data {
                    if producer.enqueue(sample).is_err() {
                        println!("err");
                    }
                }
            },
            move |err| {
                // react to errors here.
                println!("err {:?}", err)
            },
            None, // None=blocking, Some(Duration)=timeout
        )
        .expect("failed to configure input stream");

    input_stream.play().unwrap();

    eframe::run_native(
        "Audio in test",
        options,
        Box::new(move |cc| {
            let app = MyApp::new(cc, consumer);
            Ok(Box::new(app))
        }),
    )
}

use realfft::{num_complex::Complex, RealFftPlanner, RealToComplex};
use std::f32::consts::PI;
use std::sync::Arc;

const FS: usize = 48_000; // assume 48kHz sample rate
struct MyApp {
    consumer: C,
    in_data: [f32; FS],
    ptr: usize, // pointer in data
    fft_in_data: [f32; FS],
    spectrum: [Complex<f32>; FS / 2 + 1],
    r2c: Arc<dyn RealToComplex<f32>>,
    fft: Fft,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>, consumer: C) -> Self {
        // make a planner
        let mut real_planner = RealFftPlanner::<f32>::new();

        // create a FFT
        let r2c = real_planner.plan_fft_forward(FS);

        Self {
            consumer,
            in_data: [0.0; FS],
            ptr: 0,
            fft_in_data: [0.0; FS],
            spectrum: [0.0.into(); FS / 2 + 1],
            r2c,
            fft: Fft::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            while let Some(s) = self.consumer.dequeue() {
                self.in_data[self.ptr] = s;
                self.ptr = (self.ptr + 1) % FS;
            }

            ui.label(format!("ptr {}", self.ptr));

            self.fft_in_data[..self.ptr].copy_from_slice(&self.in_data[FS - self.ptr..]);
            self.fft_in_data[self.ptr..].copy_from_slice(&self.in_data[..FS - self.ptr]);

            // self.fft_in_data[1024..].copy_from_slice(&[0.0; FS - 1024]);

            self.r2c
                .process(&mut self.fft_in_data, &mut self.spectrum)
                .unwrap();

            self.fft.ui_content(ui, &self.spectrum);

            ctx.request_repaint();
        });
    }
}

#[derive(Default)]
struct Fft {}

impl Fft {
    pub fn ui_content(&mut self, ui: &mut Ui, fft_data: &[Complex<f32>]) -> egui::Response {
        let size = ui.available_size();
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        let rect = response.rect;
        trace!("rect {:?}", rect);

        let fft_stroke = Stroke::new(1.0, Color32::from_gray(255));

        let mut max = 0.0;
        for i in 0..rect.width() as usize {
            if fft_data[i].norm() > max {
                max = fft_data[i].norm()
            }
        }

        // draw spectrum
        for i in 0..rect.width() as usize {
            painter.vline(
                i as f32 + rect.left(),
                Rangef::new(
                    rect.top() + rect.height() * (1.0 - fft_data[i].norm() / max),
                    rect.bottom(),
                ),
                fft_stroke,
            );
        }

        // painter.debug_rect(rect, Color32::RED, "here");
        response
    }
}
