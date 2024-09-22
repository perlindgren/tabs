//
use num::Complex;
use std::f32::consts::PI;

pub fn gen_filter(fs: usize, f_expected: f32, p: u8) -> Vec<Complex<f32>> {
    let ws = (p as f32 * fs as f32 / f_expected) as i32; //window size
    let w = -ws / 2..=ws / 2; // inclusive range

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

    sin_cos_hann
}
