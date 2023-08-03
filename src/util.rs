pub(crate) fn read_mac(name: &str) -> eyre::Result<Vec<u8>> {
    Ok(name.split(':')
        .map(|x| u8::from_str_radix(x, 16))
        .collect::<Result<Vec<u8>, _>>()?)
}

pub(crate) fn format_mac(bytes: &[u8]) -> String {
    let result: Vec<_> = bytes.iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    result.join(":")
}
