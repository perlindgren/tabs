use cpal::{traits::*, *};
use heapless::spsc::*;

use egui::*;
use log::*;

const QUEUE_SIZE: usize = 1024; // in f32
type Q = Queue<f32, { QUEUE_SIZE * 2 }>;
type C = Consumer<'static, f32, { QUEUE_SIZE * 2 }>;

// const FS: usize = 48_000; // assume 48kHz sample rate
const FS: usize = 48_000; // assume 48kHz sample rate

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
        sample_rate: SampleRate(FS as u32),
        buffer_size: BufferSize::Fixed(64 * 4), // 64 samples
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
use std::sync::Arc;

struct MyApp {
    consumer: C,
    in_data: [f32; FS],
    ptr: usize, // pointer in data
    r2c: Arc<dyn RealToComplex<f32>>,
    fft: Fft,
}

const NR_FFTS: usize = 2;

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>, consumer: C) -> Self {
        // make a planner
        let mut real_planner = RealFftPlanner::<f32>::new();

        // let mut r2cs = vec![];

        // for i in 0..NR_FFTS {
        //     let fft_size = FS as u32 / 2u32.pow(i as u32);
        //     // create a FFT
        //     r2cs.push(real_planner.plan_fft_forward(fft_size as usize));
        // }

        let r2c = real_planner.plan_fft_forward(FS);

        Self {
            consumer,
            in_data: [0.0; FS],
            ptr: 0,
            r2c,
            fft: Fft::default(),
        }
    }
}

// in_data     [0,1, .., ptr, ..., FS-1]
//                 newest | oldest ...
// fft_in_data [oldest           newest]

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            while let Some(s) = self.consumer.dequeue() {
                self.in_data[self.ptr] = s; // most resent sample
                self.ptr = (self.ptr + 1) % FS; // next
            }

            ui.label(format!("ptr {}", self.ptr));

            let mut spectrums = vec![];

            let mut fft_in_data = self.r2c.make_input_vec();
            let mut spectrum = self.r2c.make_output_vec();

            for r2c in 1..=NR_FFTS {
                // make a dummy real-valued signal (filled with zeros)

                // make a vector for storing the spectrum

                fft_in_data[r2c.len() - self.ptr..].copy_from_slice(&self.in_data[..self.ptr]);
                fft_in_data[..r2c.len() - self.ptr].copy_from_slice(&self.in_data[self.ptr..]);

                // most recent sample
                assert_eq!(
                    self.in_data[(FS + self.ptr - 1) % FS],
                    *fft_in_data.last().unwrap()
                );
                // oldest sample
                assert_eq!(self.in_data[self.ptr], *fft_in_data.first().unwrap());

                // // TODO: apply some window function
                r2c.process(&mut fft_in_data, &mut spectrum).unwrap();
                spectrums.push(spectrum);
            }

            self.fft.ui_content(ui, spectrums);

            ctx.request_repaint();
        });
    }
}

struct Fft {
    // old_norm: [f32; FS / 2 + 1],
}

impl Default for Fft {
    fn default() -> Self {
        Fft {
            // old_norm: [0.0; FS / 2 + 1],
        }
    }
}

impl Fft {
    pub fn ui_content(&mut self, ui: &mut Ui, spectrums: Vec<Vec<Complex<f32>>>) -> egui::Response {
        // ui.label(format!("max {:?}", self.max));

        let mut size = ui.available_size();
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        let rect = response.rect;
        trace!("rect {:?}", rect);

        let fft_strokes = [
            Stroke::new(1.0, Color32::from_gray(255)),
            Stroke::new(1.0, Color32::YELLOW),
        ];
        for spectrum in spectrums {
            // size.x = size.x.max(fft_data.len() as f32);

            // draw spectrum
            for (i, bin) in spectrum.iter().take(rect.width() as usize).enumerate() {
                painter.vline(
                    i as f32 + rect.left(),
                    Rangef::new(
                        rect.top()
                            + rect.height() * (1.0 - bin.norm() / (spectrum.len() as f32).sqrt()),
                        rect.bottom(),
                    ),
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