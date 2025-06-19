# ESP-Hosted-MCU: Protocol description

This document describes the wire protocol used by [ESP-Hosted-MCU](https://github.com/espressif/esp-hosted-mcu).
It is current as of version 2.0.8.

All byte encodings are little endian.

## Frame structure

### Bytes 0-11: Payload header
The first twelve bytes are a payload header common to all messages.

- **0, bits 0:4**: Interface type: An `8-bit enum` with values like Serial, AP, and HCI.
- **0, bits 4:8**: Interface number. 0 is a good default.
- **1:** flags. 0 is a good default.
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
- **22, 23:** Payload len: `u16`. This is the length, in bytes, of all remaining data after this TLV header. (Bytes 24-)


### Remaining bytes: RPC data
The remaining data is specific to the request or response type, and is structured according to the RPC protocol.
It uses variable-length integers (_varints_). The data is organized as follows, with no spacing between items. 

The `Tag` type is used several types. It's a varint-encoded enum of two values: (field <<3) | (wire_type as u8). 
Field is a `u8`starting at 1. Wire type is an enum as follows:

```rust
 enum WireType {
    /// 0: int32, uint32, bool, enum, variant
    Basic = 0,
    /// 64-bit fixed
    SixtyFourBit = 1,
    /// Len-determined (string, bytes, sub-message)
    LenDetermined = 2,
    /// 32-bit fixed (fixed32, float)
    Fixed32Bit = 5,
}
```

Frame data continued, starting at byte 24 (All varints, except the payload).

- **24**: Tag for field 1, wire type 0; always 8.
- **25**: RPC message type: (e.g. 1 for Request, 2 for Response.)
- **26**: Tag for field 2, wire type 0; always 16.
- **27, 28**: RPC ID, encoded as a varint. This defines the nature of the request.
- **29**: Tag for field 3, wire type 0; always 24.
- **30-**: UID; a unique identifier of the requester's choice.
- **next**: Tag with field = the message id, and wire type 2 (Length determined).
- **next**: Data length (The RPC-id-specific payload that follows)
- **next**: Data, as required.


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


## Example composing a simple frame
We will demonstrate sending a frame that requests a list of Wi-Fi Access points. The full frame we send, 34-bytes long:

`[3, 0, 22, 0, 12, 0, 30, 4, 21, 0, 0, 0, 1, 6, 0, 82, 80, 67, 82, 115, 112, 2, 10, 0, 8, 1, 16, 183, 2, 24, 0, 186, 19, 0]`

Break down:

### Payload header
This is generic, and will be similar for all messages you send and receive. Set payload length and checksum as
required. Increment sequence number.

`[3, 0, 22, 0, 12, 0, 30, 4, 21, 0, 0, 0]`
- 
- **3:** Interface type = 3 (Serial). Interface number = 0.
- **0:** No flags
- **22, 0:** Payload len, following this header, of 22. (12 + 22 = 34 byte frame size)
- **12, 0:** Payload header size = 12 for offset
- **30, 4** Checksum
- **21, 0**: Sequence number. (You may wish to increment this each message you send)
- **0, 0**: Throttle 0, and no relevant packet type.


### TLV header
Other than RPC length, this is generic.

`[1, 6, 0, 82, 80, 67, 82, 115, 112, 2, 10, 0]`

- **1:** Always
- **6, 0:** Always
- **82, 80, 67, 82, 115, 112**: b"RPCRsp"
- **2:** Always
- **10, 0** 10 bytes remain in the frame, following this TLV header.


### RPC data
This is the non-generic part of our message. In this example, requesting WiFi stations.
Note that all numerical values here are varint-encoded.

`[8, 1, 16, 183, 2, 24, 0, 186, 19, 0]`

- **8:**: Tag for field 1, wire type 0; always this.
- **1**: RPC message type: Request.
- **16**: Tag for field 2, wire type 0; always this.
- **183, 2**: RPC ID = 311, encoded as varint.
- **24**: Tag for field 3, wire type 0; always this.
- **0**: UID; a unique identifier of the requester's  choice, used to track the response.

- **186, 19** Tag field = 311 (The RPC ID), wire type 2 (Length determined)
- **0** Data len encoded as varint.


Hmm:
```proto
enum RpcType {
MsgType_Invalid = 0;
	Req = 1;
	Resp = 2;
	Event = 3;
	MsgType_Max = 4;
}

message Rpc_Req_WifiApGetStaList {
}
```


Hmm again:
This is the `Rpc` the post-TLV payload is decoding into: