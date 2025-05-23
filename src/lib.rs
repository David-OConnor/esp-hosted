#![no_std]
#![allow(dead_code)]

//! An interface for interacting with ESP-Hosted-MCU firmware, via UART.
//! todo: SPI support A/R for higher data rates.
//!
//! ┌──────────────── 1. 12-byte PayloadHeader ───────────────┐
//! │  if_type / if_num          (packed nibbles)             │
//! │  flags                                                │
//! │  len   = 6   ← length of the RPC header that follows  │
//! │  offset = 0                                          │
//! │  checksum = Σ(payload bytes)  ← here: Σ(RpcHeader)    │
//! │  seq_num (host side)                                 │
//! │  throttle_cmd / rsvd                                 │
//! │  pkt_type = ESP_PACKET_TYPE_EVENT (0x33)             │
//! └─────────────────────────────────────────────────────────┘
//! ┌──────────────── 2. 6-byte RpcHeader ────────────────────┐
//! │  magic   = 0xEC                                        │
//! │  version = 1                                           │
//! │  length  = 0   ← ping has no payload                   │
//! │  module  = Module::Ctrl (0)                            │
//! │  command = PingReq  (0x01)                             │
//! └─────────────────────────────────────────────────────────┘
//! ┌──────────────── 3. (optional) payload — none here ─────┐
//! └─────────────────────────────────────────────────────────┘
//! ┌──────────────── 4. 16-bit CRC-16/CCITT-FALSE ──────────┐
//! │  polynomial 0x1021, init 0xFFFF                        │
//! │  **little-endian** on the wire       │
//! │  CRC covers *everything* from version (byte 1) of the  │
//! │  RPC header up to the last payload byte                │
//! └─────────────────────────────────────────────────────────┘

mod rf;
// mod misc;


#[macro_export]
macro_rules! parse_le {
    ($bytes:expr, $t:ty, $range:expr) => {{ <$t>::from_le_bytes($bytes[$range].try_into().unwrap()) }};
}

macro_rules! copy_le {
    ($dest:expr, $src:expr, $range:expr) => {{ $dest[$range].copy_from_slice(&$src.to_le_bytes()) }};
}


use core::sync::atomic::{AtomicBool, Ordering};

use crc_any::{CRCu8, CRCu16};
use defmt::println;
use hal::{
    pac::{SPI2, USART1, USART2},
    spi::{Spi, SpiError},
    usart::{UartError, Usart},
};
use heapless::{String, Vec};
use num_enum::{IntoPrimitive, TryFromPrimitive};

// todo: Allow any uart.
type Uart = Usart<USART2>;
// type Uart = Usart<USART1>;

// todo: How can we make this flexible? EH?

const MAGIC: u8 = 0xEC;
const VERSION: u8 = 1;
const RPC_HEADER_SIZE: usize = 6;
pub const PL_HEADER_SIZE: usize = 12;
const CRC_LEN: usize = 2;

const AP_BUF_MAX: usize = 100;
const BLE_BUF_MAX: usize = 100;

const ESP_ERR_HOSTED_BASE: u16 = 0x2f00;

const MORE_FRAGMENT: u8 = 1 << 0; // todo type and if we use it.

/// A simple error enum for our host-side protocol
#[derive(Debug)]
pub enum EspError {
    Uart(UartError),
    UnexpectedResponse(u8),
    CrcMismatch,
    Timeout,
    // todo: etc. as needed
}

impl From<UartError> for EspError {
    fn from(e: UartError) -> Self {
        EspError::Uart(e)
    }
}

/// The CRC emebedded in the Payload Header.
pub fn crc_pl_header(buf: &[u8], pl_end: usize) -> u16 {
    // todo: Dedicated function for computing checksum.
    buf[PL_HEADER_SIZE + 2 .. pl_end]
        .iter()
        .fold(0u16, |acc, &b| acc.wrapping_add(b as u16))
}

/// The CRC at the end of the frame.
/// Calculate CCITT-FALSE over [ver..]  (i.e. skip the magic byte)
fn crc_frame(buf: &[u8]) -> u16 {
    let mut crc = CRCu16::crc16ccitt_false();
    crc.digest(&buf[1..]); // start at Version
    crc.get_crc()
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum Command {
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
enum PacketType {
    /* data-path kinds — rarely used on the control UART */
    WifiData = 0x00, // 802.3 / 802.11 data via netif
    // BtHci      = 0x01,  // HCI ACL / EVT
    // todo: Oh my. What actually are these?
    HciCommand = 0x01,
    HciAclData = 0x02,
    HciScoData = 0x03,
    HciEvent = 0x04,
    HciIsoData = 0x05,
    // SerialPty  = 0x02,  // AT/PTY stream
    /* … other data codes, e.g. SCO = 0x03 … */
    /* private / control events */
    Event = 0x33, // ← what goes in Ping/Pong, scan results, etc.
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
/// Interface type.
pub enum IfType {
    // todo: How to serialize?
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

// todo: DO I need this SLIP-encoding? UART-only, if so.
/// SLIP-encode into `out`, return number of bytes written.
fn slip_encode(src: &[u8], out: &mut [u8]) -> usize {
    const END: u8 = 0xC0;
    const ESC: u8 = 0xDB;
    const ESC_END: u8 = 0xDC;
    const ESC_ESC: u8 = 0xDD;

    let mut w = 0;
    out[w] = END;          // flush garbage on the line
    w += 1;

    for &b in src {
        match b {
            END => { out[w] = ESC; out[w + 1] = ESC_END; w += 2; }
            ESC => { out[w] = ESC; out[w + 1] = ESC_ESC; w += 2; }
            _        => { out[w] = b; w += 1; }
        }
    }
    out[w] = END;
    w + 1              // trailing END
}


/// Adapted from `esp-hosted-mcu/common/esp_hosted_header.h`
/// This is at the start of the message, and is followed by the RPC header.
struct PayloadHeader {
    pub if_type: IfType, // 2 4-bit values
    pub if_num: u8,      // 2 4-bit values
    pub flags: u8,
    pub len: u16,
    pub offset: u16,
    pub checksum: u16,
    pub seq_num: u16,
    pub throttle_cmd: u8, // 2 4-bit values
    // u8 reserved; `reserve2:6`
    pub pkt_type: PacketType,
}


impl PayloadHeader {
    pub fn new(if_type: IfType, pkt_type: PacketType, rpc_bytes: &[u8], payload: &[u8]) -> Self {
        let event_hdr   = [pkt_type as u8, rpc_bytes.len() as u8]; // 0x33, 0x06

        let rpc_checksum = event_hdr.iter()
            .chain(rpc_bytes)
            .chain(payload)
            .fold(0u16, |c, &b| c.wrapping_add(b as u16));

        Self {
            if_type,
            if_num: 0,
            flags: 0,
            len: (CRC_LEN + rpc_bytes.len() + payload.len()) as u16,
            offset: 0,
            checksum: rpc_checksum,
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
        // buf[10] = (self.throttle_cmd & 0x03)
        //     | ((self.reserved2 & 0x3F) << 2);

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

        let throttle_cmd = buf[10] & 0x03;
        let reserved2 = (buf[10] >> 2) & 0x3F;
        let pkt_type = buf[11].try_into().unwrap_or(PacketType::WifiData);

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

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Module {
    /// “system / housekeeping”
    Ctrl = 0x00,
    Wifi = 0x01,
    Ble = 0x02,
}

/// This header is followed by a payload of 0-1024 bytes. (Transport agnostic). then
/// a CRC-16/CCITT-false. Polynomial: 0x1021. Init: 0xffff. The CRC is calcualted over
/// everything except the magic byte. (Version through end of payload.
///
/// This is preceded by a Payload leader.
struct RpcHeader {
    pub magic: u8,   // Always 0xEC.
    pub version: u8, // Always 1, for now.
    /// Payload length, not including CRC. LE.
    pub length: u16,
    pub module: Module,
    pub command: Command,
}

impl RpcHeader {
    pub fn new(module: Module, command: Command, payload_len: usize) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            length: payload_len as u16,
            module,
            command,
        }
    }

    pub fn to_bytes(&self) -> [u8; RPC_HEADER_SIZE] {
        let length = self.length.to_le_bytes();
        [
            self.magic,
            self.version,
            length[0],
            length[1],
            self.module as u8,
            self.command as u8,
        ]
    }

    pub fn from_bytes(buf: &[u8]) -> Self {
        Self {
            magic: buf[0],
            version: buf[1],
            length: parse_le!(buf, u16, 2..4),
            module: Module::try_from_primitive(buf[4]).unwrap(),
            command: Command::try_from_primitive(buf[5]).unwrap(),
        }
    }
}

/// Frame structure:
/// Bytes 0..12: Payload header.
/// Bytes 12..18: Payload
/// Bytes 18..18 + payload len: Payload
/// Trailing 2 bytes: CRC (frame; different from the payload header CRC)
fn build_frame<'a>(
    out: &mut [u8],
    rpc_mod: Module,
    rpc_cmd: Command,
    payload: &[u8],
) {
    let payload_len = payload.len();

    let rpc_header = RpcHeader::new(rpc_mod, rpc_cmd, payload_len);
    let rpc_bytes = rpc_header.to_bytes();

    let payload_header = PayloadHeader::new(IfType::Priv, PacketType::Event, &rpc_bytes, &payload);
    out[..PL_HEADER_SIZE].copy_from_slice(&payload_header.to_bytes());

    out[PL_HEADER_SIZE..PL_HEADER_SIZE + RPC_HEADER_SIZE].copy_from_slice(&rpc_header.to_bytes());

    let pl_end = PL_HEADER_SIZE + RPC_HEADER_SIZE + payload_len;
    out[PL_HEADER_SIZE + RPC_HEADER_SIZE..pl_end].copy_from_slice(payload);

    // todo: Dedicated function for computing checksum.
    // let checksum = out[PL_HEADER_SIZE + 2 .. pl_end]
    //     .iter()
    //     .fold(0u16, |acc, &b| acc.wrapping_add(b as u16));
    // out[6..8].copy_from_slice(&checksum.to_le_bytes());

    let crc = crc_frame(&out[PL_HEADER_SIZE + 3 .. pl_end]); // skip magic
    copy_le!(out, crc, pl_end..pl_end + CRC_LEN);
}

/// Round-trip health-check.  Returns Err on timeout / CRC / protocol error.
pub fn status_check(uart: &mut Uart, timeout_ms: u32) -> Result<(), EspError> {
    const MAX_FRAME: usize = PL_HEADER_SIZE + 2 + RPC_HEADER_SIZE + CRC_LEN;

    let mut frame_buf = [0u8; MAX_FRAME];
    let frame_len =
        build_frame(&mut frame_buf, Module::Ctrl, Command::PingReq, &[]);

    // uart.write(frame_buf)?;

    // todo: Experimenting with slip_buf. wrap this in a helper if required.
    let mut slip_buf = [0u8; 2 * MAX_FRAME + 2]; // worst-case expansion
    let slip_len = slip_encode(&frame_buf[..frame_len], &mut slip_buf);
    uart.write(&slip_buf[..slip_len])?;

    println!("Writing status check frame: {:?}", &frame_buf);

    // --------- receive header ---------
    let mut hdr = [0; RPC_HEADER_SIZE];

    // uart.read_exact_timeout(&mut hdr, timeout_ms)?;
    uart.read(&mut hdr)?;

    println!("Header buf read: {:?}", hdr);

    if hdr[0] != MAGIC || hdr[1] != VERSION {
        println!("ESP Unexpected magic or version"); // todo temp
        return Err(EspError::UnexpectedResponse(hdr[0]));
    }
    let len = u16::from_le_bytes([hdr[2], hdr[3]]) as usize;

    // --------- receive payload + CRC ---------
    let mut rest = [0; 1_026]; // more than enough for empty payload + CRC
    // uart.read_exact_timeout(&mut rest[..len + CRC_LEN], timeout_ms)?;
    uart.read(&mut rest[..len + CRC_LEN])?;

    // validate CRC
    let mut full = [0u8; RPC_HEADER_SIZE + 1_026];
    full[..RPC_HEADER_SIZE].copy_from_slice(&hdr);
    full[RPC_HEADER_SIZE..RPC_HEADER_SIZE + len].copy_from_slice(&rest[..len]);

    let rx_crc = u16::from_le_bytes(rest[len..len + CRC_LEN].try_into().unwrap());
    if crc_frame(&full[..RPC_HEADER_SIZE + len]) != rx_crc {
        println!("ESP CRC mismatch"); // todo temp
        return Err(EspError::CrcMismatch);
    }

    // validate that it is indeed a PingResp
    if hdr[4] != Module::Ctrl as u8 || hdr[5] != Command::PingResp as u8 {
        println!("ESP Unexpected resp"); // todo temp
        return Err(EspError::UnexpectedResponse(hdr[5]));
    }

    Ok(())
}
