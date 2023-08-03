use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use eyre::{bail, Context, eyre};
use ini::{Ini, Properties};
use crate::cmd::DUMP_FILE;
use crate::model::{Adapter, BLEDeviceCreds, BytesAsMACWrapper, DataDump, Device, DeviceCreds, LongTermKey, RegularDeviceCreds};
use crate::util::read_mac;

const BT_ROOT_DIR: &str = "/var/lib/bluetooth";
const INVALID_DEVICE_NAMES: &[&str] = &[
    "cache",
    "settings"
];

pub(super) fn main() -> eyre::Result<()> {
    println!("Reading '{BT_ROOT_DIR}'...");
    let result = dump_all()?;

    println!("Writing data to '{DUMP_FILE}'...");
    let serialized = serde_json::to_string(&result)?;
    fs::write(DUMP_FILE, &serialized)?;

    println!("OK!");
    Ok(())
}

fn dump_all() -> eyre::Result<DataDump> {
    let bt_root = PathBuf::from(BT_ROOT_DIR);

    let adapters = bt_root.read_dir()?;
    let mut out = HashMap::new();
    for adapter in adapters {
        let adapter = adapter?;
        let file_name = adapter.file_name();
        let file_name = file_name.to_str()
            .ok_or_else(|| eyre!("failed to read: {adapter:?}"))?;
        let adapter_mac = read_mac(file_name)
            .with_context(|| eyre!("failed to parse MAC address: {file_name}"))?;

        out.insert(BytesAsMACWrapper(adapter_mac), dump_adapter(&adapter.path())?);
    }

    Ok(DataDump { adapters: out })
}

fn dump_adapter(adapter_path: &Path) -> eyre::Result<Adapter> {
    let devices = adapter_path.read_dir()?;
    let mut out = HashMap::new();
    for device in devices {
        let device = device?;
        let file_name = device.file_name();
        let file_name = file_name.to_str()
            .ok_or_else(|| eyre!("failed to read: {device:?}"))?;

        if INVALID_DEVICE_NAMES.contains(&file_name) {
            continue
        }

        let device_mac = read_mac(file_name)
            .with_context(|| eyre!("failed to parse MAC address: {file_name}"))?;

        out.insert(BytesAsMACWrapper(device_mac), dump_device(&device.path())?);
    }

    Ok(Adapter { devices: out })
}

fn dump_device(device_path: &Path) -> eyre::Result<Device> {
    let ini = Ini::load_from_file(device_path.join("info"))?;

    let Some(general_section) = ini.section(Some("General")) else {
        bail!("device {device_path:?} is missing 'General' section");
    };
    let name = general_section.get("Name")
        .ok_or_else(|| eyre!("device {device_path:?} is missing name"))?
        .to_string();

    Ok(Device {
        name,
        creds: dump_device_creds(&ini)?
    })
}

fn dump_device_creds(ini: &Ini) -> eyre::Result<DeviceCreds> {
    Ok(if let Some(link_key_section) = ini.section(Some("LinkKey")) {
        DeviceCreds::Regular(dump_regular_device_creds(link_key_section)?)
    } else {
        DeviceCreds::BLE(dump_ble_device_creds(ini)?)
    })
}

fn dump_regular_device_creds(link_key_section: &Properties) -> eyre::Result<RegularDeviceCreds> {
    let Some(key_hex) = link_key_section.get("Key") else {
        bail!("device is missing 'Key' in LinkKey section");
    };

    Ok(RegularDeviceCreds {
        link_key: hex::decode(key_hex)?
    })
}

fn dump_ble_device_creds(ini: &Ini) -> eyre::Result<BLEDeviceCreds> {
    let Some(irk_section) = ini.section(Some("IdentityResolvingKey")) else {
        bail!("device is missing IdentityResolvingKey section");
    };
    let Some(irk_key_hex) = irk_section.get("Key") else {
        bail!("device is missing 'Key' in IdentityResolvingKey section");
    };
    let irk_key = hex::decode(irk_key_hex)
        .with_context(|| eyre!("IRK is not hex"))?;

    let ltk = ini.section(Some("LongTermKey"))
        .map(dump_ltk)
        .transpose()?;

    let peripheral_ltk = ini.section(Some("PeripheralLongTermKey"))
        .map(dump_ltk)
        .transpose()?;

    Ok(BLEDeviceCreds {
        identity_resolving_key: irk_key,
        long_term_key: ltk,
        peripheral_long_term_key: peripheral_ltk
    })
}

fn dump_ltk(section: &Properties) -> eyre::Result<LongTermKey> {
    let Some(key_hex) = section.get("Key") else {
        bail!("device is missing 'Key' in LTK section");
    };
    let key = hex::decode(key_hex)
        .context("'Key' is not hex")?;

    let Some(enc_size_str) = section.get("EncSize") else {
        bail!("device is missing 'EncSize' in LTK section");
    };
    let enc_size: u16 = enc_size_str.parse()
        .context("'EncSize' is not an integer")?;

    let Some(ediv_str) = section.get("EDiv") else {
        bail!("device is missing 'EDiv' in LTK section");
    };
    let ediv: u64 = ediv_str.parse()
        .context("'EDiv' is not an integer")?;

    let Some(rand_str) = section.get("Rand") else {
        bail!("device is missing 'Rand' in LTK section");
    };
    let rand: u64 = rand_str.parse()
        .context("'Rand' is not an integer")?;

    Ok(LongTermKey {
        key,
        enc_size,
        ediv,
        rand
    })
}