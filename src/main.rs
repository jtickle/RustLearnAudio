use cpal;
use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    println!("Available Hosts:");
    for host_id in cpal::available_hosts() {
        println!("  {}:", host_id.name());

        let host = cpal::host_from_id(host_id).unwrap();
        println!("    Input Devices:");
        for device in host.input_devices().unwrap() {
            println!("      {}:", device.name().unwrap());
        }

        println!("    Output Devices:");
        for device in host.output_devices().unwrap() {
            println!("      {}:", device.name().unwrap());
        }
    }
}