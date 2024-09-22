//
use num::Complex;
use std::f32::consts::PI;

/// sin_cos_hann
///
/// generates a filter for the desired frequency
/// fs:usize        sample frequency in Hz
/// f_expected:f32  expected frequency to analyze
/// nr_period:u8    the number of periods for window
#[inline(always)]
pub fn sin_cos_hann(fs: usize, f_expected: f32, nr_periods: u8) -> Vec<Complex<f32>> {
    let mut ws = (nr_periods as f32 * fs as f32 / f_expected) as i32; // window size
    ws += ws % 2; // make even
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
            c * c / ws as f32
        })
        .collect();

    // hanning weighted sin_cos window
    let sin_cos_hann: Vec<_> = sin_cos.iter().zip(hann).map(|(c, h)| h * c).collect();
    assert_eq!(sin_cos_hann.len(), (ws + 1) as usize);
    sin_cos_hann
}

// completely un-optimized convolution
// convolution actually starting from k + filter.len()/2
// assumes k + filter.len() < data.len()
#[inline(always)]
pub fn conv_at_k(data: &[f32], filter: &[Complex<f32>], k: usize) -> Complex<f32> {
    assert!(k + filter.len() < data.len());

    let mut sum: Complex<f32> = 0.0f32.into();
    for (i, c) in filter.iter().enumerate() {
        sum += data[k + i] * c;
    }
    sum
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_sin_cos_hann() {
        let fs = 48_000;
        let f_expected = 82.51;
        let nr_periods = 10;
        let _filter = sin_cos_hann(fs, f_expected, nr_periods);
    }

    // to benchmark:
    // cargo test test_sin_cos_hann_timed --release -- --nocapture
    //
    // expected output something like
    // sin_cos_hann (generate) = 447.166µs
    // conv_at_k (apply) = 25.964µs

    #[test]
    fn test_sin_cos_hann_timed() {
        const FS: usize = 48_000;
        let f_expected = 82.51;
        let nr_periods = 40;

        let start = Instant::now();

        let nr_test = 10_000;

        for _ in 0..nr_test {
            let _filter = sin_cos_hann(FS, f_expected, nr_periods);
        }

        let end = Instant::now();
        println!("sin_cos_hann (generate) = {:?}", (end - start) / nr_test);

        let filter = sin_cos_hann(FS, f_expected, nr_periods);

        let data = [0.0f32; FS];
        let start = Instant::now();

        for _ in 0..nr_test {
            let r = conv_at_k(&data, &filter, FS / 2);
            unsafe {
                std::ptr::read_volatile(&r);
            }
        }

        let end = Instant::now();
        println!("conv_at_k (apply) = {:?}", (end - start) / nr_test);
    }
}
