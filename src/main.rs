// Strongly inspired by, and some sections totally ripped off from:
// https://github.com/RustAudio/cpal/blob/master/examples/feedback.rs

use cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;

macro_rules! display_whatevers {
    ($fn: expr, $and_then: ident, $prefix: expr, $or_error: expr) => {
        let mut count:u32 = 0;
        $fn.expect(&$or_error)
        .for_each(|x| {
            $and_then(&x, $prefix, count);
            count += 1;
        })
    }
}

fn display_config(config: &cpal::SupportedStreamConfigRange, prefix: &str, count: u32) {
    let npfx = &format!("{}  ", prefix);
    println!("{}{}:", prefix, count);

    println!("{}Channels:        {:?}", npfx, config.channels());
    println!("{}Min Sample Rate: {:?}", npfx, config.min_sample_rate());
    println!("{}Max Sample Rate: {:?}", npfx, config.max_sample_rate());
    println!("{}Buffer Size:     {:?}", npfx, config.buffer_size());
    println!("{}Sample Format:   {:?}", npfx, config.sample_format());
}

fn display_device(device: &cpal::Device, prefix: &str, _count: u32) {
    let dev_name = device.name().unwrap();
    println!("{}{}:", prefix, dev_name);

    let npfx = &format!("{}  ", prefix);
    let npfx2 = &format!("{}    ", prefix);

    println!("{}Input Configs:", npfx);

    display_whatevers!(device.supported_input_configs(), display_config, npfx2,
        format!("Error retrieving input configs for device {}", dev_name));

    println!("{}Output Configs:", npfx);

    display_whatevers!(device.supported_output_configs(), display_config, npfx2,
            format!("Error retrieving output configs for device {}", dev_name));
}

fn display_capabilities() {
    println!("Available Hosts:");
    cpal::available_hosts().iter()
    .map(|host_id| -> (&str, cpal::Host) {
        (host_id.name(), cpal::host_from_id(*host_id).expect(&format!("Error getting host with ID {}", host_id.name())))
    })
    .for_each(|(host_name, host)| {

        println!("  {}:", host_name);

        println!("    Input Devices:");
        display_whatevers!(host.input_devices(), display_device, "      ",
            format!("Error retrieving input devices for host {}", host_name));

        println!("    Output Devices:");
        display_whatevers!(host.output_devices(), display_device, "      ",
            format!("Error retrieving output devices for host {}", host_name));
    });

}

fn main() {
    display_capabilities();

    // Define latency
    let latency:f32 = 1000.0;

    let host = cpal::default_host();
    let input_device = host.default_input_device().unwrap();
    let output_device = host.default_output_device().unwrap();

    println!("Using input device: \"{}\"", input_device.name().unwrap());
    println!("Using output device: \"{}\"", output_device.name().unwrap());

    // Try using the same configuration between streams to keep it simple
    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();

    // Create a delay in case input and output devices aren't synced
    let latency_frames = (latency / 1_000.0) * config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * config.channels as usize;

    // Create the buffer to share samples
    let ring = HeapRb::<f32>::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();

    // Fill the samples with 0.0 equal to the length of the delay.
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.push(0.0).unwrap();
    }

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let mut output_fell_behind = false;
        for &sample in data {
            if producer.push(sample).is_err() {
                output_fell_behind = true;
            }
        }
        if output_fell_behind {
            eprintln!("output stream fell behind: try increasing latency");
        }
    };

    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut input_fell_behind = false;
        for sample in data {
            *sample = match consumer.pop() {
                Some(s) => s,
                None => {
                    input_fell_behind = true;
                    0.0
                }
            };
        }
        if input_fell_behind {
            eprintln!("input stream fell behind: try increasing latency");
        }
    };

    // Build streams.
    println!(
        "Attempting to build both streams with f32 samples and `{:?}`.",
        config
    );
    let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn).unwrap();
    let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn).unwrap();
    println!("Successfully built streams.");

    // Play the streams.
    println!(
        "Starting the input and output streams with `{}` milliseconds of latency.",
        latency
    );
    input_stream.play().unwrap();
    output_stream.play().unwrap();

    // Run for 3 seconds before closing
    println!("Playing for 3 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(3));

    drop(input_stream);
    drop(output_stream);
    println!("Done!");
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}