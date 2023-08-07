use std::collections::HashMap;
use std::io;
use std::ops::Range;
use eyre::{bail, Context, ContextCompat, ensure};
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, RegDisposition, RegType};
use winreg::{RegKey, RegValue};
use winreg::transaction::Transaction;
use crate::dump::read_dump;
use crate::model::{BLEDeviceCreds, BytesAsMACWrapper, DeviceCreds, LongTermKey, RegularDeviceCreds};
use crate::util::{format_mac, read_mac, vec_take};

pub(super) fn main(adapter: &str, device: &str) -> eyre::Result<()> {
    let adapter_addr = read_mac(adapter)?;
    let device_addr = read_mac(device)?;

    let data = read_dump()?;
    let Some(adapter_data) = data.adapters.get(&BytesAsMACWrapper(adapter_addr.clone())) else {
        bail!("adapter {adapter} is not in present in data dump");
    };
    let Some(device_data) = adapter_data.devices.get(&BytesAsMACWrapper(device_addr.clone())) else {
        bail!("device {device} is not in present in data dump");
    };

    let reg_trans = Transaction::new()?;
    let check_or_suggest_addr_with_reg = |ble: bool| check_or_suggest_addr_with_reg(
        &reg_trans,
        &adapter_addr,
        &device_addr,
        &device_data.name,
        ble
    );
    match &device_data.creds {
        DeviceCreds::Regular(creds) => {
            let local_device_addr = check_or_suggest_addr_with_reg(false)?;
            apply_regular(&reg_trans, creds, &adapter_addr, &local_device_addr)?;
            // Rename device address if address is different from source device
            if local_device_addr != device_addr {
                // Move info
                move_device_info(&reg_trans, &local_device_addr, &device_addr)?;
                // Move key
                let adapter_key = open_bt_reg_key_rw(&reg_trans, &adapter_addr, None)?;
                let from_value_name = format_mac_win(&local_device_addr)?;
                let to_value_name = format_mac_win(&device_addr)?;
                adapter_key.set_raw_value(
                    to_value_name,
                    &adapter_key.get_raw_value(&from_value_name)?
                )?;
                adapter_key.delete_value(from_value_name)?;
            }
        },
        DeviceCreds::BLE(creds) => {
            let local_device_addr = check_or_suggest_addr_with_reg(true)?;
            apply_ble(&reg_trans, creds, &adapter_addr, &local_device_addr)?;
            // Rename device address if address is different from source device
            if local_device_addr != device_addr {
                // Move info
                move_device_info(&reg_trans, &local_device_addr, &device_addr)?;
                // Move keys
                let adapter_key = open_bt_reg_key_rw(&reg_trans, &adapter_addr, None)?;
                let from_value_name = format_mac_win(&local_device_addr)?;
                let to_value_name = format_mac_win(&device_addr)?;
                reg_move_subkey(&reg_trans, &adapter_key, from_value_name, to_value_name, true)?;
                // Update 'Address' value in key
                {
                    let target = open_bt_reg_key_rw(&reg_trans, &adapter_addr, Some(&device_addr))?;
                    // Convert target MAC address into u64
                    ensure!(device_addr.len() == 6, "new MAC address is invalid");
                    let device_addr_u64 = u64::from_be_bytes(array_init::from_iter(
                        [0, 0].into_iter().chain(device_addr.iter().copied())
                    ).unwrap());
                    target.set_value("Address", &device_addr_u64)?;
                }
            }
        }
    }

    reg_trans.commit()?;

    println!("Device '{device}' in adapter '{adapter}' updated!");

    Ok(())
}

// ===== APPLY =====

fn apply_regular(reg_trans: &Transaction, creds: &RegularDeviceCreds, adapter_addr: &[u8], device_addr: &[u8]) -> eyre::Result<()> {
    let adapter_key = open_bt_reg_key_rw(reg_trans, adapter_addr, None)?;

    // Ensure there is an existing value
    let encoded_device_addr = format_mac_win(device_addr)?;
    validate_reg_value(&adapter_key, &encoded_device_addr, RegType::REG_BINARY)?;

    // Set new value
    adapter_key.set_raw_value(encoded_device_addr, &RegValue {
        bytes: creds.link_key.clone(),
        vtype: RegType::REG_BINARY
    })?;

    Ok(())
}

fn apply_ble(reg_trans: &Transaction, creds: &BLEDeviceCreds, adapter_addr: &[u8], device_addr: &[u8]) -> eyre::Result<()> {
    // Open subkey
    let device_key = open_bt_reg_key_rw(reg_trans, adapter_addr, Some(device_addr))?;

    // Update IRK
    validate_reg_value(&device_key, IRK_KEY_NAME, RegType::REG_BINARY)?;
    device_key.set_raw_value(IRK_KEY_NAME, &RegValue {
        bytes: creds.identity_resolving_key.clone(),
        vtype: RegType::REG_BINARY
    })?;

    // Update LTK
    match (&creds.long_term_key, &creds.peripheral_long_term_key) {
        (Some(ltk), _) | (None, Some(ltk)) => apply_ble_ltk(&device_key, ltk)?,
        _ => bail!("device has both an LTK and a PeripheralLTK, it is not known how to handle this situation")
    }

    Ok(())
}

const IRK_KEY_NAME: &str = "IRK";
const LTK_KEY_NAME: &str = "LTK";
const EDIV_KEY_NAME: &str = "EDIV";
const ERAND_KEY_NAME: &str = "ERand";
const KEY_LENGTH_KEY_NAME: &str = "KeyLength";

fn apply_ble_ltk(device_key: &RegKey, new_ltk: &LongTermKey) -> eyre::Result<()> {
    validate_reg_value(device_key, LTK_KEY_NAME, RegType::REG_BINARY)?;
    device_key.set_raw_value(LTK_KEY_NAME, &RegValue {
        bytes: new_ltk.key.clone(),
        vtype: RegType::REG_BINARY
    })?;

    validate_reg_value(device_key, EDIV_KEY_NAME, RegType::REG_DWORD)?;
    device_key.set_value(EDIV_KEY_NAME, &new_ltk.ediv)?;

    validate_reg_value(device_key, ERAND_KEY_NAME, RegType::REG_QWORD)?;
    device_key.set_value(ERAND_KEY_NAME, &new_ltk.rand)?;

    validate_reg_value(device_key, KEY_LENGTH_KEY_NAME, RegType::REG_DWORD)?;
    device_key.set_value(KEY_LENGTH_KEY_NAME, &new_ltk.enc_size)?;

    Ok(())
}

fn open_bt_reg_key_rw(reg_trans: &Transaction, adapter: &[u8], device: Option<&[u8]>) -> eyre::Result<RegKey> {
    let encoded_adapter = format_mac_win(adapter)?;

    const BASE_PATH: &str = r#"SYSTEM\CurrentControlSet\Services\BTHPORT\Parameters\Keys"#;
    let key_path = if let Some(device) = device {
        let encoded_device = format_mac_win(device)?;
        format!(r#"{BASE_PATH}\{encoded_adapter}\{encoded_device}"#)
    } else {
        format!(r#"{BASE_PATH}\{encoded_adapter}"#)
    };

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    Ok(hklm.open_subkey_transacted_with_flags(key_path, reg_trans, KEY_READ | KEY_WRITE)?)
}

// ===== Device Info =====

const DEVICE_INFO_REG_PATH: &str = r#"SYSTEM\CurrentControlSet\Services\BTHPORT\Parameters\Devices"#;
fn get_device_info_reg_key_path(device: &[u8]) -> eyre::Result<String> {
    Ok(format!(r#"{DEVICE_INFO_REG_PATH}\{}"#, format_mac_win(device)?))
}

fn get_device_name(reg_trans: &Transaction, device: &[u8]) -> eyre::Result<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let device_key = hklm.open_subkey_transacted(
        get_device_info_reg_key_path(device)?,
        reg_trans
    )?;

    let raw_name = device_key.get_raw_value("Name")?;
    ensure!(raw_name.vtype == RegType::REG_BINARY, "'Name' value for device '{device:?}' has invalid type");

    String::from_utf8(raw_name.bytes)
        .with_context(|| format!("device '{device:?}' has a Name that is not valid UTF-8"))
}

fn move_device_info(reg_trans: &Transaction, old_device_addr: &[u8], new_device_addr: &[u8]) -> eyre::Result<()> {
    let device_info_key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_transacted_with_flags(DEVICE_INFO_REG_PATH, reg_trans, KEY_READ | KEY_WRITE)?;
    reg_move_subkey(
        reg_trans,
        &device_info_key,
        format_mac_win(old_device_addr)?,
        format_mac_win(new_device_addr)?,
        false
    )
}

// ===== Suggest =====

fn check_or_suggest_addr_with_reg(
    reg_trans: &Transaction,
    adapter_addr: &[u8],
    target_addr: &[u8],
    target_addr_name: &str,
    ble: bool
) -> eyre::Result<Vec<u8>> {
    let adapter_key = open_bt_reg_key_rw(reg_trans, adapter_addr, None)?;

    let possible_addrs: Vec<String> = if ble {
        // For BLE, devices are stored as subkeys
        adapter_key.enum_keys()
            .filter_map(Result::ok)
            .collect()
    } else {
        // For normal, devices are stored as values
        adapter_key.enum_values()
            .filter_map(Result::ok)
            .filter(|v| v.0 != "MasterIRK") // Exclude MasterIRK value
            .map(|a| a.0)
            .collect()
    };
    let possible_addrs: HashMap<Vec<u8>, String> = possible_addrs.iter()
        .filter_map(|a| parse_mac_win(a).ok())
        .filter_map(|a| get_device_name(reg_trans, &a).ok().map(|n| (a, n)))
        .collect();

    check_or_suggest_addr(target_addr, target_addr_name, possible_addrs)
}

fn check_or_suggest_addr(
    target_addr: &[u8],
    target_addr_name: &str,
    mut possible_addrs: HashMap<Vec<u8>, String>,
) -> eyre::Result<Vec<u8>> {
    // Is the MAC paired to the system?
    if let Some((addr, _)) = possible_addrs.remove_entry(target_addr) {
        return Ok(addr);
    }

    // No, find similar MACs that might correspond to the device
    // Only keep MACs that have the same first 3 bytes (OUI) as target MAC. The device is allowed to randomize the NIC.
    const OUI_IDXS: Range<usize> = 0..3;
    let Some(target_oui) = target_addr.get(OUI_IDXS) else {
        bail!("target MAC address is invalid")
    };
    let possible_addrs: Vec<(Vec<u8>, String)> = possible_addrs.into_iter()
        .filter(|(possible, _)| possible.get(OUI_IDXS) == Some(target_oui))
        .collect();

    // No similar MACs
    if possible_addrs.is_empty() {
        bail!("{target_addr_name} ({}) is not paired on this system", format_mac(target_addr));
    }

    // Show similar MACs
    println!("{target_addr_name} ({}) is not paired on this system. \
    Some Bluetooth devices slightly randomize/change their MAC address when they get re-paired, \
    here is a list of devices with similar MAC addresses paired to this system:", format_mac(target_addr));
    for (idx, (possible_addr, name)) in possible_addrs.iter().enumerate() {
        println!("\t[{}] {} => {name}", idx + 1, format_mac(possible_addr))
    }

    // Ask user for MAC
    println!("==> Enter the index of the MAC address that corresponds to this device (leave blank to cancel):");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input: usize = input.trim().parse().context("invalid index")?;

    Ok(vec_take(possible_addrs, input - 1)
        .context("invalid index")?.0)
}

// ===== Utils =====

fn validate_reg_value(key: &RegKey, value_name: &str, expected_type: RegType) -> eyre::Result<()> {
    let existing_value = key.get_raw_value(value_name)?;
    ensure!(existing_value.vtype == expected_type, "existing value is not valid");
    Ok(())
}

// ===== MAC Address Utils =====

const MAC_ADDR_WIDTH: usize = 6;
const MAC_ADDR_WIN_WIDTH: usize = MAC_ADDR_WIDTH * 2; // 2 chars for each byte in hex notation

/// This is method is required to properly format MAC address that have leading zeros
fn format_mac_win(mac: &[u8]) -> eyre::Result<String> {
    if mac.len() != MAC_ADDR_WIDTH {
        bail!("invalid MAC address: {mac:?}")
    }
    Ok(format!("{:0>MAC_ADDR_WIN_WIDTH$}", hex::encode(mac)))
}
fn parse_mac_win(mac: &str) -> eyre::Result<Vec<u8>> {
    let decoded = hex::decode(mac)?;
    if decoded.len() != MAC_ADDR_WIDTH {
        bail!("invalid MAC address: {decoded:?}");
    }
    Ok(decoded)
}

// ===== Registry Utils =====
/// Move subkey from `from` to `to`. If `overwrite` is `false` and `to` already exists, `from` is simply deleted and `to` is left unchanged.
///
/// **`reg_key` MUST be transacted (so don't pass a `predef` key!)**
fn reg_move_subkey(reg_trans: &Transaction, reg_key: &RegKey, from: impl AsRef<str>, to: impl AsRef<str>, overwrite: bool) -> eyre::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    // Open or create the registry key for the new device info
    let (to_key, open_result) = reg_key.create_subkey_transacted(to, reg_trans)?;

    // Copy device info
    if overwrite || open_result == RegDisposition::REG_CREATED_NEW_KEY {
        reg_key.copy_tree(from, &to_key)?;
    }

    // Delete old device info
    reg_key.delete_subkey_all(from)?;

    Ok(())
}