// Rust re-implementation of frequency matched filters

use num::complex::ComplexFloat;
use rodio::{Decoder, Source};
use std::{fs::File, path::Path};

fn main() {
    let e2_file = File::open(Path::new("./e2.wav")).unwrap();
    let f2_file = File::open(Path::new("./f2.wav")).unwrap();

    let source_e2 = Decoder::new(e2_file).unwrap();
    let source_f2 = Decoder::new(f2_file).unwrap();
    let fs = source_e2.sample_rate() as usize;
    let samples_e2: Vec<f32> = source_e2.convert_samples::<f32>().collect();
    let samples_f2: Vec<f32> = source_f2.convert_samples::<f32>().collect();

    let samples_e2 = samples_e2.as_slice();
    let samples_f2 = samples_f2.as_slice();

    // % 1 = 1st harmonic, fundamental
    // % 2 = 2nd harmonic, one octave
    // % 3 = 3nd harmonic, one octave + and a fifth
    // % 4 = 4th harmonic, two octaves
    // % 5 = 5th harmonic, two octaves + a major third
    // % etc
    let harmonic: u8 = 4;

    let f_expected = 82.41 * harmonic as f32; // frequency to analyze, low E guitar

    let p: u8 = 40; // number of periods
    let ws = (p as f32 * fs as f32 / f_expected) as i32; //window size

    let latency_time = ws as f32 / fs as f32;
    let latency_samples = ws;

    println!(
        "latency_time {}, latency_samples {}",
        latency_time, latency_samples
    );
    // hanning weighted sin_cos window
    let sin_cos_hann = tabs::dsp::sin_cos_hann(fs, f_expected, p);

    println!("sin_cos_hann {}", sin_cos_hann.first().unwrap());

    let c = tabs::dsp::conv_at_k(&samples_f2, &sin_cos_hann, fs / 2);
    println!("c {:?}", c.abs());

    let c2 = tabs::dsp::conv_at_k(&samples_e2, &sin_cos_hann, fs / 2);
    println!("c2 {:?}", c2.abs());
}
