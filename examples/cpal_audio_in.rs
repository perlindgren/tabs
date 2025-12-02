use cpal::{traits::*, *};
use egui::old_popup;
use heapless::spsc::*;

use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

use log::*;
use std::sync::{Arc, Mutex};
use tabs::spectrum::SpectrumView;

const QUEUE_SIZE: usize = 2048; // in f32
type Q = Queue<f32, { QUEUE_SIZE * 2 }>;
type C = Consumer<'static, f32>;

const FS: usize = 48_000; // assume 48kHz sample rate

struct SampleBuffer {
    ptr: usize,
    data: [f32; FS],
}

impl Default for SampleBuffer {
    fn default() -> Self {
        Self {
            ptr: 0,
            data: [0.0; FS],
        }
    }
}

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

    let (mut producer, mut consumer) = spsc.split();

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

    let mtx_sample_buffer = Arc::new(Mutex::new(SampleBuffer::default()));

    let sb_writer = Arc::clone(&mtx_sample_buffer);

    let _jh = std::thread::spawn(move || {
        loop {
            // lock mutex
            let mut sb = sb_writer.lock().unwrap();

            let mut ptr = sb.ptr;
            while let Some(s) = consumer.dequeue() {
                sb.data[ptr] = s; // most recent sample
                ptr = (ptr + 1) % FS; // next
            }
            sb.ptr = ptr; // update pointer
            let _ = drop(sb); // unlock mutex
            println!("ptr {}, now {:?}", ptr, std::time::SystemTime::now());

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let app = MyApp::new(mtx_sample_buffer);

    eframe::run_native(
        "Audio in test",
        options,
        Box::new(move |cc| Ok(Box::new(app))),
    )
}

struct MyApp {
    fft: SpectrumView,
    mtx_sample_buffer: Arc<Mutex<SampleBuffer>>,
}

impl MyApp {
    fn new(mtx_sample_buffer: Arc<Mutex<SampleBuffer>>) -> Self {
        Self {
            mtx_sample_buffer,
            fft: SpectrumView::default(),
        }
    }
}

// in_data     [0,1, .., ptr, ..., FS-1]
//                 newest | oldest ...
// fft_in_data [oldest           newest]

// const WINDOW: usize = FS;
const WINDOW: usize = 32768;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let sb = self.mtx_sample_buffer.lock().unwrap();
            ui.label(format!("ptr {}", sb.ptr));

            // create
            let mut fft_in_data = [0.0; FS];

            fft_in_data[FS - sb.ptr..].copy_from_slice(&sb.data[..sb.ptr]);
            fft_in_data[..FS - sb.ptr].copy_from_slice(&sb.data[sb.ptr..]);

            // most recent sample
            assert_eq!(
                sb.data[(FS + sb.ptr - 1) % FS],
                *fft_in_data.last().unwrap()
            );
            // oldest sample
            assert_eq!(sb.data[sb.ptr], *fft_in_data.first().unwrap());

            let mut spectrums = vec![];
            drop(sb); // unlock mutex

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
