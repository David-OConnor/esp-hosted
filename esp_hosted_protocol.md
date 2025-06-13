# ESP-Hosted-MCU: Protocol description

This document describes the wire protocol used by [ESP-Hosted-MCU](https://github.com/espressif/esp-hosted-mcu).
It is current as of version 2.0.8.

All byte encodings are little endian.

## Frame structure

### Bytes 0-11: Payload header
The first twelve bytes are a payload header common to all messages.

- **0, bits 0:4**: Interface type: An `8-bit enum` with values like Serial, AP, and HCI.
- **0, bits 4:8**: Interface number. 0 is a good default.
- **1:** flags
- **2, 3:** Payload length: `u16`. The size, in bytes, of everything in the frame following this header.
- **4, 5**: Offset: `u16`. **Always = [12, 0]** (This header's size). Indicates the byte index the payload starts.
- **6, 7:** Checksum, calculated over the entire frame; see below. Initialize it to 0, then compute at the end.
- **8, 9:** `u16`. Sequence number for tracking packets (Useful in debugging)
- **10, bits 0:2:** Throttle command. 0 is a good default.
- **11:** Packet type. Default = 0. For HCI and Private packet types, this takes the value  `0x33` and `0x22` respectively.


### Bytes 12-23: TLV header
The TLV (type, length, value) header further describes _endpoint_ and length data.

- **12:** `u8`. Always `1` to indicate the following bytes specify the Endpoint name.
- **13, 14:** Length of the endpoint name below. `u16.` **Always [6, 0]**.
- **15 - 20** Endpoint name. Hosts always writes."RPCRsp" as ASCII bytes. Slave may write that, or "RPCEvt".
- **21:** `u8.` Always `2` to indicate the following bytes are data.
- **22, 23:** Payload len: `u16`. This is the length, in bytes, of all remaining data.


## Checksum computation
See`System_design_with_rps_as_focus.md`, section 3.3: Checksum Calculation.
```rust
fn compute_checksum(buf: &[u8]) -> u16 {
    let mut checksum = 0;
    let mut i = 0;

    while i < buf.len() {
        checksum += buf[i] as u16;
        i += 1;
    }

    checksum
}
```