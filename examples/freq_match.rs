// Rust re-implementation of frequency matched filters

use num::{complex::ComplexFloat, Complex};
use std::f32::consts::PI;

fn main() {
    let fs: usize = 48000;
    let x = 0..fs;

    // % 1 = 1st harmonic, fundamental
    // % 2 = 2nd harmonic, one octave
    // % 3 = 3nd harmonic, one octave + and a fifth
    // % 4 = 4th harmonic, two octaves
    // % 5 = 5th harmonic, two octaves + a major third
    // % etc
    let harmonic: u8 = 5;

    let f_expected = 82.41 * harmonic as f32; // frequency to analyze, low E guitar

    let p: u8 = 40; // number of periods
    let ws = (p as f32 * fs as f32 / f_expected) as i32; //window size

    let latency_time = ws as f32 / fs as f32;
    let latency_samples = ws;

    println!(
        "latency_time {}, latency_samples {}",
        latency_time, latency_samples
    );

    let w = -ws / 2..=ws / 2; // inclusive range

    let f = f_expected;
    let data: Vec<f32> = x
        .clone()
        .map(|k| (2.0 * PI * f * k as f32 / fs as f32).sin())
        .collect();

    let f2 = 87.31 * harmonic as f32;
    let data2: Vec<f32> = x
        .map(|k| (2.0 * PI * f2 * k as f32 / fs as f32).sin())
        .collect();

    // complex representation
    let sin_cos: Vec<Complex<_>> = w
        .clone()
        .map(|k| {
            let v = 2.0 * PI * f_expected * k as f32 / fs as f32;
            Complex::new(v.sin(), v.cos())
        })
        .collect();

    let hann: Vec<_> = w
        .clone()
        .map(|k| {
            let v = k as f32 * PI / ws as f32;
            let c = v.cos();
            if k == 0 {
                println!("c {}", c)
            };
            c * c / ws as f32
        })
        .collect();

    println!(
        "hann {}, first {}, mid {}, last {}, ws min weighted {}",
        hann.len(),
        hann.first().unwrap(),
        hann.get(hann.len() / 2).unwrap(),
        hann.last().unwrap(),
        ws as f32 * hann.get(hann.len() / 2).unwrap(),
    );

    // hanning weighted sin_cos window
    let sin_cos_hann: Vec<_> = sin_cos.iter().zip(hann).map(|(c, h)| h * c).collect();

    println!("sin_cos_hann {}", sin_cos_hann.first().unwrap());

    let c = conv_at_k(&data, &sin_cos_hann, fs / 2);
    println!("c {:?}", c.abs());

    let c2 = conv_at_k(&data2, &sin_cos_hann, fs / 2);
    println!("c2 {:?}", c2.abs());
}

// completely un-optimized convolution
// convolution actually starting from k + filter.len()/2
// assumes k + filter.len() < data.len()
fn conv_at_k(data: &[f32], filter: &[Complex<f32>], k: usize) -> Complex<f32> {
    assert!(k + filter.len() < data.len());

    let mut sum: Complex<f32> = 0.0f32.into();
    for (i, c) in filter.iter().enumerate() {
        sum += data[k + i] * c;
    }
    sum
}
