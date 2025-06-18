/// Encodes `v` as little-endian 7-bit var-int.
/// Returns number of bytes written (1–3 for a `u16`).
pub(crate) fn encode_varint(mut v: u64, out: &mut [u8]) -> usize {
    let mut idx = 0;
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            out[idx] = byte; // last byte – high bit clear
            idx += 1;
            break;
        } else {
            out[idx] = byte | 0x80; // more bytes follow
            idx += 1;
        }
    }
    idx
}

/// Decodes a little-endian 7-bit var-int.
/// Returns `(value, bytes_consumed)`.
pub(crate) fn decode_varint(input: &[u8]) -> (u64, usize) {
    let mut val = 0u64;
    let mut shift = 0;
    for (idx, &byte) in input.iter().enumerate() {
        val |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return (val, idx + 1);
        }
        shift += 7;
    }
    panic!("unterminated var-int");
}
