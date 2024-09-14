use cpal::{traits::*, *};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("no input device available");
    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: SampleRate(48000),
        buffer_size: BufferSize::Fixed(64),
    };

    // The buffer to share samples
    let ring = HeapRb::<f32>::new(64 * 2);
    let (mut producer, mut consumer) = ring.split();

    let input_stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], info: &cpal::InputCallbackInfo| {
                // react to stream events and read or write stream data here.
                println!("data len {}, info {:?}", data.len(), info);

                for &sample in data {
                    if producer.try_push(sample).is_err() {
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

    loop {
        let a = consumer.try_pop();
    }
}
