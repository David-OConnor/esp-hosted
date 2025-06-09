//! This module contains the basic protocol used by all messages.

use num_enum::TryFromPrimitive;
use crate::{copy_le, parse_le};

pub(crate) const MAGIC: u8 = 0xEC;
pub(crate) const VERSION: u8 = 1;
pub(crate) const TLV_HEADER_SIZE: usize = 6;
pub(crate) const PL_HEADER_SIZE: usize = 12; // Verified from ESP docs
pub(crate) const CRC_LEN: usize = 2;

/// Compute CRCs; for by the one embedded in the payload header, and at the end of the message.
/// The `buf` argument for this function must already by set to the correct range.
///
/// Payload header: Covers Entire ESP frame from byte 0 of the 12-byte payload header up to – but not including – the
/// two-byte trailer CRC that terminates the frame.
/// In other words: header + RPC header + payload.
///
/// Frame (trailing) CRC: Starts with the version field of the RPC header
/// (i.e. it skips the constant magic byte 0xEC) and continues up to the last byte of the
/// payload.
pub(crate) fn calc_crc(buf: &[u8]) -> u16 {
    buf.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as _))
}

// /// The CRC at the end of the frame.
// /// Calculate CCITT-FALSE over [ver..]  (i.e. skip the magic byte)
// /// CRC-16/CCITT-false. Polynomial: 0x1021. Init: 0xffff. The `buf` argument
// /// for this function must already by set to the correct range.
// fn crc_frame(buf: &[u8]) -> u16 {
//     let mut crc = CRCu16::crc16ccitt_false();
//     crc.digest(&buf[1..]); // start at Version
//     crc.get_crc()
// }

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub(crate) enum Command {
    // ---- Control ----
    /// empty payload
    PingReq = 0x01,
    /// empty payload – reply to PingReq
    PingResp = 0x02,

    // ---- Wi-Fi ----
    /// Empty payload
    WifiScanStart = 0x10,
    /// variable payload, multiple frames
    WifiScanResult = 0x11,

    // ---- BLE ----
    /// Empty payload
    BleScanStart = 0x20,
    /// Variable payload, multiple frames
    BleScanResult = 0x21,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub(crate) enum PacketTypeHci{
    A = 0,
    B = 1, // todo
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
/// Private
pub(crate) enum PacketTypePriv{
    A = 0,
    B = 1, // todo
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PacketType {
    // todo??
    Hci(PacketTypeHci),
    Priv(PacketTypePriv),

    // /* data-path kinds — rarely used on the control UART */
    // WifiData = 0x00, // 802.3 / 802.11 data via netif
    // // BtHci      = 0x01,  // HCI ACL / EVT
    // // todo: Oh my. What actually are these?
    // HciCommand = 0x01,
    // HciAclData = 0x02,
    // HciScoData = 0x03,
    // HciEvent = 0x04,
    // HciIsoData = 0x05,
    // // SerialPty  = 0x02,  // AT/PTY stream
    // /* … other data codes, e.g. SCO = 0x03 … */
    // /* private / control events */
    // Event = 0x33, // ← what goes in Ping/Pong, scan results, etc.
}

impl PacketType {
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::Hci(p) => *p as u8,
            Self::Priv(p) => *p as u8,
        }
    }

    pub fn from_byte(val: u8) -> Self {
        // todo temp; fix.
       Self::Priv(PacketTypePriv::A)
    }
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
/// Interface type. See ESP-Hosted-MCU readme, section 7.2
/// Embassy shows different Implied to be the same in ESP doc, but not explicitly. Esp proto:
pub(crate) enum IfType {
    Invalid = 0,
    Stat = 1,
    Ap = 2,
    Serial = 3,
    Hci = 4,
    Priv = 5,
    Test = 6,
    Eth = 7,
    Max = 8

    // Embassy:
    // Sta = 0,
    // Ap = 1,
    // Serial = 2,
    // Hci = 3,
    // Priv = 4,
    // Test = 5,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub(crate) enum Module {
    /// “system / housekeeping”
    Ctrl = 0x00,
    Wifi = 0x01,
    Ble = 0x02,
}

// todo: DO I need this SLIP-encoding? UART-only, if so.
/// SLIP-encode into `out`, return number of bytes written.
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
///
/// todo: This checks with both embassy, and the WIP Espressif doc.
struct PayloadHeader {
    /// Interface type
    pub if_type: IfType, // 2 4-bit values
    /// Interface number
    pub if_num: u8,      // 2 4-bit values
    pub flags: u8,
    /// Payload length
    pub len: u16,
    /// Offset to payload
    pub offset: u16,
    /// Header + payload checksum
    pub checksum: u16,
    /// Sequence number for tracking packets (Useful in debugging)
    pub seq_num: u16,
    /// Flow control
    pub throttle_cmd: u8, // First two bits of this byte.
    // u8 reserved; `reserve2:6`; same byte as `throttle_cmd`

    // From Esp doc: First 3 bits may be reserved. The remaining bits for HCI or PRivate packet type?
    pub pkt_type: PacketType,
}

impl PayloadHeader {
    pub fn new(if_type: IfType, pkt_type: PacketType, payload_len: usize) -> Self {
        Self {
            if_type,
            if_num: 0,
            flags: 0,
            len: (TLV_HEADER_SIZE + payload_len + CRC_LEN) as u16,
            offset: 0,
            checksum: 0, // Computed after the entire frame is constructed.
            seq_num: 0,
            throttle_cmd: 0,
            pkt_type,
        }
    }
    /// Serialize into the 12-byte packed representation
    pub fn to_bytes(&self) -> [u8; PL_HEADER_SIZE] {
        let mut buf = [0; PL_HEADER_SIZE];

        // byte 0:   [ if_num:4 | if_type:4 ]
        buf[0] = ((self.if_type as u8) << 4) | (self.if_num & 0x0F);
        buf[1] = self.flags;

        copy_le!(buf, self.len, 2..4);
        copy_le!(buf, self.offset, 4..6);
        copy_le!(buf, self.checksum, 6..8);
        copy_le!(buf, self.seq_num, 8..10);

        // byte 10:  [ reserved2:6 | throttle_cmd:2 ]
        buf[10] = self.throttle_cmd; // todo: QC if you need a shift.

        // byte 11: union field
        buf[11] = self.pkt_type.to_byte();

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
        let pkt_type = PacketType::from_byte(buf[11]);

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


// /// Type-length-value header. See host/drivers/virtual_serial_if/serial_if.c
// struct TlvHeader {
//     pub magic: u8,   // Always 0xEC.
//     pub version: u8, // Always 1, for now.
//     /// Payload length, not including CRC. LE.
//     pub length: u16,
//     pub module: Module,
//     pub command: Command,
// }

// impl TlvHeader {
//     pub fn new(module: Module, command: Command, payload_len: usize) -> Self {
//         Self {
//             magic: MAGIC,
//             version: VERSION,
//             length: payload_len as u16,
//             module,
//             command,
//         }
//     }
//
//     pub fn to_bytes(&self) -> [u8; TLV_HEADER_SIZE] {
//         let length = self.length.to_le_bytes();
//         [
//             self.magic,
//             self.version,
//             length[0],
//             length[1],
//             self.module as u8,
//             self.command as u8,
//         ]
//     }
//
//     pub fn from_bytes(buf: &[u8]) -> Self {
//         Self {
//             magic: buf[0],
//             version: buf[1],
//             length: parse_le!(buf, u16, 2..4),
//             module: Module::try_from_primitive(buf[4]).unwrap(),
//             command: Command::try_from_primitive(buf[5]).unwrap(),
//         }
//     }
// }

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Default)]
#[repr(u8)]
/// Type-length-value header. See host/drivers/virtual_serial_if/serial_if.c
pub(crate) enum TlvType {
    #[default]
    EndpointName = 0x01,
    Data = 0x02,
}

// #[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
// #[repr(u8)]
// // todo
// pub(crate) enum DataType {
//     A = 0
// }

/// Type-length-value header. See host/drivers/virtual_serial_if/serial_if.c
struct TlvHeader {
    pub endpoint_type: TlvType,
    pub endpoint_length: u16,
    // pub endpoint_value: &[u8], // endpoint length
    pub data_type: TlvType,
    pub data_length: u16,
    // pub data_value: &[u8], // data length
}

impl TlvHeader {
    pub fn to_bytes(&self, buf: &mut [u8], endpoint_value: &[u8], data_value: &[u8])  {
        buf[0] = self.endpoint_type as u8;
        copy_le!(buf, self.endpoint_length, 1..3);

        let mut i = 3;
        buf[i..i + self.endpoint_length as usize].copy_from_slice(endpoint_value);
        i += self.endpoint_length as usize;

        buf[i] = self.data_type as u8;
        i += 1;

        copy_le!(buf, self.data_length, i..i + 2);
        i += 2;

        buf[i..i + self.data_length as usize].copy_from_slice(data_value);
    }

    pub fn from_bytes(buf: &[u8]) -> Self {
        // todo: Where do we read the values?
        // todo: Return a Result for failed enum parsing?

        let endpoint_length = parse_le!(buf, u16, 1..3);
        let epl = endpoint_length as usize;

        Self {
            endpoint_type: buf[0].try_into().unwrap_or_default(),
            endpoint_length,
            data_type: buf[3 + epl].try_into().unwrap_or_default(),
            data_length: parse_le!(buf, u16, 3 + epl..3 + epl + 2),
        }
    }
}

/// Frame structure:
/// Bytes 0..12: Payload header.
/// Bytes 12..18: Payload
/// Bytes 18..18 + payload len: Payload
/// Trailing 2 bytes: CRC (frame; different from the payload header CRC)
pub(crate) fn build_frame(out: &mut [u8], rpc_mod: Module, rpc_cmd: Command, payload: &[u8]) {
    let payload_len = payload.len();

    let tlv_header = TlvHeader::new(rpc_mod, rpc_cmd, payload_len);

    // let payload_header = PayloadHeader::new(IfType::Priv, PacketType::Event, payload_len);
    let payload_header = PayloadHeader::new(IfType::Priv, PacketType::Priv(PacketTypePriv::A), payload_len);
    out[..PL_HEADER_SIZE].copy_from_slice(&payload_header.to_bytes());

    out[PL_HEADER_SIZE..PL_HEADER_SIZE + TLV_HEADER_SIZE].copy_from_slice(&tlv_header.to_bytes());

    let pl_end = PL_HEADER_SIZE + TLV_HEADER_SIZE + payload_len;
    out[PL_HEADER_SIZE + TLV_HEADER_SIZE..pl_end].copy_from_slice(payload);

    // Now that the frame is constructed (Except for the trailing CRC), compute the payload CRC.
    let pl_checksum = calc_crc(&out[..pl_end]);
    copy_le!(out, pl_checksum, 6..8);

    // TLV start (except magic) through payload end.
    let frame_crc = calc_crc(&out[PL_HEADER_SIZE + 1..pl_end]);
    copy_le!(out, frame_crc, pl_end..pl_end + CRC_LEN);
}
