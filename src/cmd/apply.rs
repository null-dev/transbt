use crate::util::read_mac;

pub(super) fn main(adapter: &str, device: &str) -> eyre::Result<()> {
    let parsed_adapter = read_mac(adapter)?;
    let encoded_adapter = hex::encode(parsed_adapter);

    todo!()
}
