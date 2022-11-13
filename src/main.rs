use cpal;
use cpal::traits::{DeviceTrait, HostTrait};

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
}