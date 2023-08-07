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

/// Take a single item from a `Vec`, discarding the rest of the elements
pub(crate) fn vec_take<T>(mut vec: Vec<T>, index: usize) -> Option<T> {
    if index < vec.len() {
        Some(vec.swap_remove(index))
    } else {
        None
    }
}