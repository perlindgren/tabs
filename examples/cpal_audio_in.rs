use cpal::{traits::*, *};
use heapless::spsc::*;

use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

use log::*;
use tabs::spectrum_view::SpectrumView;

const QUEUE_SIZE: usize = 1024; // in f32
type Q = Queue<f32, { QUEUE_SIZE * 2 }>;
type C = Consumer<'static, f32, { QUEUE_SIZE * 2 }>;

// const FS: usize = 48_000; // assume 48kHz sample rate
const FS: usize = 2048 * 2; // assume 48kHz sample rate

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
        #[allow(static_mut_refs)]
        unsafe {
            &mut SPSC
        }
    };

    let (mut producer, consumer) = spsc.split();

    let input_stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _info: &cpal::InputCallbackInfo| {
                for &sample in data {
                    if producer.enqueue(sample).is_err() {
                        println!("spsc queue full");
                    }
                }
            },
            move |err| {
                // react to errors here.
                println!("stream error {:?}", err)
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

struct MyApp {
    consumer: C,
    in_data: [f32; FS],
    ptr: usize, // pointer in data
    fft: SpectrumView,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>, consumer: C) -> Self {
        Self {
            consumer,
            in_data: [0.0; FS],
            ptr: 0,
            fft: SpectrumView::default(),
        }
    }
}

// in_data     [0,1, .., ptr, ..., FS-1]
//                 newest | oldest ...
// fft_in_data [oldest           newest]

const WINDOW: usize = FS;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            while let Some(s) = self.consumer.dequeue() {
                self.in_data[self.ptr] = s; // most resent sample
                self.ptr = (self.ptr + 1) % FS; // next
            }

            ui.label(format!("ptr {}", self.ptr));

            // create
            let mut fft_in_data = [0.0; FS];

            fft_in_data[FS - self.ptr..].copy_from_slice(&self.in_data[..self.ptr]);
            fft_in_data[..FS - self.ptr].copy_from_slice(&self.in_data[self.ptr..]);

            // most recent sample
            assert_eq!(
                self.in_data[(FS + self.ptr - 1) % FS],
                *fft_in_data.last().unwrap()
            );
            // oldest sample
            assert_eq!(self.in_data[self.ptr], *fft_in_data.first().unwrap());

            let mut spectrums = vec![];

            for i in 0..4 {
                // spectrum analysis only of the latest WINDOW
                let relevant_samples = &fft_in_data[fft_in_data.len() - WINDOW / 2usize.pow(i)..];

                // do FFT
                let hann_window = hann_window(relevant_samples);
                let spectrum = samples_fft_to_spectrum(
                    &hann_window,
                    FS as u32,
                    FrequencyLimit::All, //
                    // FrequencyLimit::Max(2000.0),
                    Some(&divide_by_N),
                )
                .unwrap();
                spectrums.push(spectrum)
            }

            self.fft.ui_content(ui, spectrums);

            ctx.request_repaint();
        });
    }
}
