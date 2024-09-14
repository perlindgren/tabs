// Computes a forward FFT of size 4096
use rand::prelude::*;
use realfft::RealFftPlanner;
use std::f32::consts::PI;

fn main() {
    const FS: usize = 48000;
    let mut data = [0.0f32; FS];

    let mut rng = rand::thread_rng();
    let y: f64 = rng.gen();
    println!("y {}", y);

    for (i, d) in data.iter_mut().enumerate() {
        *d = ((41.2 * 2.0 * PI * i as f32 + 1.5 * PI) / (FS as f32)).sin() * 0.05
            + if i > 000 {
                ((43.7 * 2.0 * PI * i as f32) / (FS as f32)).sin() * 0.1
                    + ((987.7 * 2.0 * PI * i as f32) / (FS as f32)).sin() * 0.05
            } else {
                0.0
            }
            + (2.0 * rng.gen::<f32>() - 1.0); //  * 0.1 * 1.0 / 32.0;
    }

    let length = 48000;

    // make a planner
    let mut real_planner = RealFftPlanner::<f32>::new();

    // create a FFT
    let r2c = real_planner.plan_fft_forward(FS);
    // make input and output vectors
    let mut indata = r2c.make_input_vec();

    // for (i, d) in indata.iter_mut().enumerate() {
    //     *d = ((41.2 * 2.0 * PI * i as f32 + 0.4 * PI) / (length as f32)).sin() * 0.5
    //         + ((43.7 * 2.0 * PI * i as f32) / (length as f32)).sin() * 0.5;
    // }
    let mut spectrum = r2c.make_output_vec();

    // Are they the length we expect?
    // assert_eq!(indata.len(), length);
    assert_eq!(spectrum.len(), length / 2 + 1);

    // Forward transform the input data
    // r2c.process(&mut indata, &mut spectrum).unwrap();
    let i = std::time::Instant::now();
    for _ in 0..1000 {
        indata = data.to_vec();
        r2c.process(&mut indata, &mut spectrum).unwrap();
    }
    let d = std::time::Instant::now() - i;
    println!("d {:?}", d);

    for (i, d) in spectrum.iter().enumerate() {
        if (i > 30 && i < 50) || (i > 970 && i < 1000) {
            println!("i {}, re {}", i, d.norm());
        }
    }
}
