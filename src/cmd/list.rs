use std::fs::OpenOptions;
use crate::cmd::DUMP_FILE;
use crate::model::DataDump;
use crate::util::format_mac;

pub(super) fn main() -> eyre::Result<()> {
    println!("Reading data dump from '{DUMP_FILE}'...\n");
    let file = OpenOptions::new()
        .read(true)
        .open(DUMP_FILE)?;

    let data: DataDump = serde_json::from_reader(file)?;
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