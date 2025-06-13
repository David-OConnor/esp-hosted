//! This module contains the payload header and TLV structure which proceeds RPC data,
//! creating a frame, and support types.

use core::sync::atomic::{AtomicU16, Ordering};

use num_enum::TryFromPrimitive;

use crate::{
    copy_le, parse_le,
    transport::{RPC_EP_NAME_RSP, compute_checksum},
};
use crate::rpc::{EndpointType, RpcEndpoint};
use crate::transport::PacketType;

pub(crate) const PL_HEADER_SIZE: usize = 12; // Verified from ESP docs

// The 6 static bytes in the TLV header: endpoint type (1), endpoint length (2), data type (1),
// data length (2).
const TLV_HEADER_SIZE: usize = 6;
// RPC_EP_NAME_EVT is the same size as `RPC_EP_NAME_RESP`.
pub(crate) const TLV_SIZE: usize = TLV_HEADER_SIZE + RPC_EP_NAME_RSP.len();

// Worst-case size:  tag(1) + 2-byte varint + tag(1) + 3-byte varint = 7
pub(crate) const RPC_HEADER_MAX_SIZE: usize = 7;

pub(crate) const CRC_SIZE: usize = 2; // todo: Determine if you need this; for trailing CRC.

pub(crate) const HEADER_SIZE: usize = PL_HEADER_SIZE + TLV_SIZE;

static SEQ_NUM: AtomicU16 = AtomicU16::new(0);

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
/// See ESP-Hosted-MCU readme, section 7.2
/// /// [official enum](https://github.com/espressif/esp-hosted-mcu/blob/634e51233af2f8124dfa8118747f97f8615ea4a6/common/esp_hosted_interface.h)
pub(crate) enum InterfaceType {
    Invalid = 0,
    Sta = 1,
    Ap = 2,
    Serial = 3,
    Hci = 4,
    Priv = 5,
    Test = 6,
    Eth = 7,
    Max = 8,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
// todo: Verify the provenance.
pub(crate) enum Module {
    /// “system / housekeeping”
    Ctrl = 0x00,
    Wifi = 0x01,
    Ble = 0x02,
}

// todo: DO I need this SLIP-encoding? UART-only, if so.
/// SLIP-encode into `out`, return number of bytes written.
/// // todo: Verify the provenance.
pub(crate) fn slip_encode(src: &[u8], out: &mut [u8]) -> usize {
    const END: u8 = 0xC0;
    const ESC: u8 = 0xDB;
    const ESC_END: u8 = 0xDC;
    const ESC_ESC: u8 = 0xDD;

    let mut w = 0;
    out[w] = END; // flush garbage on the line
    w += 1;

    for &b in src {
        match b {
            END => {
                out[w] = ESC;
                out[w + 1] = ESC_END;
                w += 2;
            }
            ESC => {
                out[w] = ESC;
                out[w + 1] = ESC_ESC;
                w += 2;
            }
            _ => {
                out[w] = b;
                w += 1;
            }
        }
    }
    out[w] = END;
    w + 1 // trailing END
}

/// Adapted from `esp-hosted-mcu/common/esp_hosted_header.h`
/// This is at the start of the message, and is followed by the RPC header.
/// See ESP-hosted-MCU readme, section 7.1.
struct PayloadHeader {
    /// Interface type. Serial, AP etc.
    pub if_type: InterfaceType, // 2 4-bit values
    /// Interface number. 0 may be a good default?
    pub if_num: u8, // 2 4-bit values
    pub flags: u8,
    /// Payload length. The size, in bytes, of everything in the frame following this
    /// header
    pub len: u16,
    ///  Offset. Always = 12 (This header's size). Indicates the byte index the payload
    /// starts.
    pub offset: u16,
    /// Checksum, calculated over the entire frame.
    pub checksum: u16,
    /// Sequence number for tracking packets (Useful in debugging)
    pub seq_num: u16,
    /// Flow control
    pub throttle_cmd: u8, // First two bits of this byte.
    // u8 reserved; `reserve2:6`; same byte as `throttle_cmd`

    // From Esp doc: First 3 bits may be reserved. The remaining bits for HCI or
    // Private packet type?

    pub pkt_type: PacketType,
}

impl PayloadHeader {
    pub fn new(if_type: InterfaceType, pkt_type: PacketType, payload_len: usize) -> Self {
        // Len is the number of bytes following the header. (all)
        let len = (TLV_SIZE + payload_len) as u16;

        Self {
            if_type,
            // todo: should we pass if_num as a param? 0 to start?
            if_num: 0,
            flags: 0,
            len,
            offset: PL_HEADER_SIZE as u16,
            // Computed after the entire frame is constructed. Must be set to 0 for
            // now, as this goes into the checksum calculation.
            checksum: 0,
            seq_num: SEQ_NUM.fetch_add(1, Ordering::SeqCst),
            throttle_cmd: 0,
            pkt_type,
        }
    }
    /// Serialize into the 12-byte packed representation
    pub fn to_bytes(&self) -> [u8; PL_HEADER_SIZE] {
        let mut buf = [0; PL_HEADER_SIZE];

        // byte 0:   [ if_num:4 | if_type:4 ]
        buf[0] = (self.if_num << 4) | ((self.if_type as u8) & 0x0F);

        buf[1] = self.flags;

        copy_le!(buf, self.len, 2..4);
        copy_le!(buf, self.offset, 4..6);
        copy_le!(buf, self.checksum, 6..8);
        copy_le!(buf, self.seq_num, 8..10);

        // byte 10:  [ reserved2:6 | throttle_cmd:2 ]
        buf[10] = self.throttle_cmd; // todo: QC if you need a shift.

        // byte 11: union field
        buf[11] = self.pkt_type as u8;

        buf
    }

    /// Parse from a 12-byte slice (will panic if `buf.len() < 12` or slice-to-array fails)
    pub fn from_bytes(buf: &[u8]) -> Self {
        let if_type = (buf[0] & 0x0F).try_into().unwrap();
        let if_num = (buf[0] >> 4) & 0x0F;
        let flags = buf[1];

        let len = parse_le!(buf, u16, 2..4);
        let offset = parse_le!(buf, u16, 4..6);
        let checksum = parse_le!(buf, u16, 6..8);
        let seq_num = parse_le!(buf, u16, 8..10);

        let throttle_cmd = buf[10] & 3;
        let pkt_type = buf[11].try_into().unwrap();

        Self {
            if_type,
            if_num,
            flags,
            len,
            offset,
            checksum,
            seq_num,
            throttle_cmd,
            pkt_type,
        }
    }
}


/// Builds the entire frame sent and received over the wire protocol. See `esp_hosted_protocol.md`
/// for details on how this is constructed.
/// Outputs total bytes in the frame.
pub(crate) fn build_frame(
    out: &mut [u8],
    payload: &[u8],
) -> usize{
    // `payload` here is all remaining bytes, including RPC metadata.
    let payload_len = payload.len();

    // From `serial_if.c`: Always Resp for compose. Either Resp or Event from parse. (host-side)
    let endpoint_value = RpcEndpoint::CtrlResp.as_bytes();
    let endpoint_len = endpoint_value.len() as u16;

    // For sending from host, always use PRIV_EVENT_INIT.
    // let packet_type = PacketType::ESP_PRIV_EVENT_INIT;
    let packet_type = PacketType::None;

    let payload_header = PayloadHeader::new(
        // todo: Serial? Sta? What should IfType be?
        InterfaceType::Serial,
        packet_type,
        payload_len,
    );
    out[..PL_HEADER_SIZE].copy_from_slice(&payload_header.to_bytes());

    let mut i = PL_HEADER_SIZE;

    out[i] = EndpointType::EndpointName as _;
    i += 1;

    copy_le!(out, endpoint_len, i..i + 2);
    i += 2;

    out[i..i + endpoint_len as usize].copy_from_slice(endpoint_value);
    i += endpoint_len as usize;

    out[i] = EndpointType::Data as _;
    i += 1;

    copy_le!(out, payload_len as u16, i..i + 2);
    i += 2;

    out[i..i + payload_len].copy_from_slice(payload);
    i += payload_len;

    // system_design...: "**Checksum Coverage**: The checksum covers the **entire frame** including:
    // 1. Complete `esp_payload_header` (with checksum field set to 0 during calculation)
    // 2. Complete payload data"
    let pl_checksum = compute_checksum(&out[..i]);
    copy_le!(out, pl_checksum, 6..8);

    i
}
