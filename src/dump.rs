use std::fs::OpenOptions;
use crate::model::DataDump;

pub const DUMP_FILE: &str = "dump.json";

pub(crate) fn read_dump() -> eyre::Result<DataDump> {
    println!("Reading data dump from '{DUMP_FILE}'...\n");
    let file = OpenOptions::new()
        .read(true)
        .open(DUMP_FILE)?;

    Ok(serde_json::from_reader(file)?)
}