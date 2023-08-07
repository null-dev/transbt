use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::util::{format_mac, read_mac};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataDump {
    pub adapters: HashMap<BytesAsMACWrapper, Adapter>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Adapter {
    pub devices: HashMap<BytesAsMACWrapper, Device>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub creds: DeviceCreds
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceCreds {
    Regular(RegularDeviceCreds),
    BLE(BLEDeviceCreds)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegularDeviceCreds {
    pub link_key: Vec<u8>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BLEDeviceCreds {
    pub identity_resolving_key: Vec<u8>,
    pub long_term_key: Option<LongTermKey>,
    pub peripheral_long_term_key: Option<LongTermKey>, // Used to be called SlaveLongTermKey
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LongTermKey {
    pub key: Vec<u8>,
    pub enc_size: u32,
    pub ediv: u32,
    pub rand: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[serde(from = "BytesAsMAC", into = "BytesAsMAC")]
pub struct BytesAsMACWrapper(pub Vec<u8>);

#[derive(Serialize, Deserialize)]
struct BytesAsMAC(String);
impl From<BytesAsMACWrapper> for BytesAsMAC {
    fn from(value: BytesAsMACWrapper) -> Self {
        Self(format_mac(&value.0))
    }
}
impl From<BytesAsMAC> for BytesAsMACWrapper {
    fn from(value: BytesAsMAC) -> Self {
        BytesAsMACWrapper(read_mac(&value.0).unwrap())
    }
}
