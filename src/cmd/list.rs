use crate::dump::{read_dump};
use crate::util::format_mac;

pub(super) fn main() -> eyre::Result<()> {
    let data = read_dump()?;

    println!("ADAPTERS:");

    for (adapter_hex, adapter) in data.adapters {
        let adapter_mac = format_mac(&adapter_hex.0);
        println!("{adapter_mac} =>");

        for (device_hex, device) in adapter.devices {
            let device_mac = format_mac(&device_hex.0);
            println!("\t{device_mac} => {}", &device.name);
        }
    }

    Ok(())
}