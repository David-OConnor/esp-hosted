#![no_std]
#![allow(dead_code)]

//! An interface for interacting with ESP-Hosted-MCU firmware, via UART.
//! todo: SPI support A/R for higher data rates.

use core::sync::atomic::{AtomicBool, Ordering};

use crc_any::{CRCu8, CRCu16};
use hal::{
    pac::{USART2, SPI2},
    usart::{UartError, Usart},
    spi::{Spi, SpiError},
};
use heapless::{String, Vec};
use num_enum::{IntoPrimitive, TryFromPrimitive};

// todo: Allow any uart.
type Uart = Usart<USART2>;

// todo: How can we make this flexible? EH?

const MAGIC: u8 = 0xEC;
const VERSION: u8 = 1;
const HEADER_SIZE: usize = 6;
const CRC_LEN: usize = 2;

pub const PAYLOAD_HEADER_SIZE: usize = 12;

const AP_BUF_MAX: usize = 100;
const BLE_BUF_MAX: usize = 100;

const ESP_ERR_HOSTED_BASE: u16 = 0x2f00;

const MORE_FRAGMENT: u8 = 1 << 0; // todo type and if we use it.

/// Calculate CCITT-FALSE over [ver..]  (i.e. skip the magic byte)
fn calc_crc(buf: &[u8]) -> u16 {
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
    Hci = 1,
    Private = 2, // todo: QC these.
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

/// Adapted from `esp-hosted-mcu/common/esp_hosted_header.h`
struct PayloadHeader {
    pub if_type: IfType, // 2 4-bit values
    pub if_num: u8,      // 2 4-bit values
    pub flags: u8,
    pub len: u16,
    pub offset: u16,
    pub checksum: u16,
    pub seq_num: u16,
    pub throttle_cmd: u8, // 2 4-bit values.
    // u8 reserved; `reserve2:6`
    pub pkt_type: PacketType,
}

impl PayloadHeader {
    /// Serialize into the 12-byte packed representation
    pub fn to_bytes(&self) -> [u8; PAYLOAD_HEADER_SIZE] {
        let mut buf = [0; PAYLOAD_HEADER_SIZE];

        // byte 0:   [ if_num:4 | if_type:4 ]
        buf[0] = (self.if_type as u8 & 0x0F) | ((self.if_num & 0x0F) << 4);

        buf[1] = self.flags;

        buf[2..4].copy_from_slice(&self.len.to_le_bytes());
        buf[4..6].copy_from_slice(&self.offset.to_le_bytes());
        buf[6..8].copy_from_slice(&self.checksum.to_le_bytes());
        buf[8..10].copy_from_slice(&self.seq_num.to_le_bytes());

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
        let len = u16::from_le_bytes(buf[2..4].try_into().unwrap());
        let offset = u16::from_le_bytes(buf[4..6].try_into().unwrap());
        let checksum = u16::from_le_bytes(buf[6..8].try_into().unwrap());
        let seq_num = u16::from_le_bytes(buf[8..10].try_into().unwrap());
        let throttle_cmd = buf[10] & 0x03;
        let reserved2 = (buf[10] >> 2) & 0x3F;
        let pkt_type = buf[11].try_into().unwrap_or(PacketType::Hci);

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

/// This header is followed by a payload of 0-1024 bytes. (Transport agnostic)., then
/// a CRC-16/CCITT-false. Polynomial: 0x1021. Init: 0xffff. The CRC is calcualted over
/// everything except the magic byte. (Version through end of payload.
struct RpcHeader {
    pub magic: u8,   // Always 0xEC.
    pub version: u8, // Always 1, for noww.
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

    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
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
            length: u16::from_le_bytes(buf[2..4].try_into().unwrap()),
            module: Module::try_from_primitive(buf[4]).unwrap(),
            command: Command::try_from_primitive(buf[5]).unwrap(),
        }
    }
}

fn build_frame<'a>(out: &'a mut [u8], module: Module, cmd: Command, payload: &[u8]) -> &'a [u8] {
    let payload_len = payload.len();
    let header = RpcHeader::new(module, cmd, payload_len);

    out[0..HEADER_SIZE].copy_from_slice(&header.to_bytes());

    let end = HEADER_SIZE + payload_len;
    out[HEADER_SIZE..end].copy_from_slice(payload);

    let crc = calc_crc(&out[..end]);
    out[end..end + 2].copy_from_slice(&crc.to_le_bytes());

    &out[..end + 2]
}

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

/// Information about one Wi-Fi access point
#[derive(Debug)]
pub struct ApInfo {
    pub ssid: String<32>,
    pub bssid: [u8; 6],
    pub rssi: i8,
}

/// Information about one BLE advertisement
#[derive(Debug)]
pub struct BleDevice {
    pub addr: [u8; 6],
    pub data: Vec<u8, 31>,
    pub rssi: i8,
}

/// Round-trip health-check.  Returns Err on timeout / CRC / protocol error.
pub fn status_check(uart: &mut Uart, timeout_ms: u32) -> Result<(), EspError> {
    // --------- send PING ---------
    let mut tx_buf = [0u8; HEADER_SIZE + CRC_LEN]; // zero-payload
    let frame = build_frame(&mut tx_buf, Module::Ctrl, Command::PingReq, &[]);
    uart.write(frame)?; // stm32-hal2 write helper

    // todo?
    // uart.flush()?;                         // ensure it’s gone

    // --------- receive header ---------
    let mut hdr = [0u8; HEADER_SIZE];

    // uart.read_exact_timeout(&mut hdr, timeout_ms)?;
    uart.read(&mut hdr)?;

    if hdr[0] != MAGIC || hdr[1] != VERSION {
        return Err(EspError::UnexpectedResponse(hdr[0]));
    }
    let len = u16::from_le_bytes([hdr[2], hdr[3]]) as usize;

    // --------- receive payload + CRC ---------
    let mut rest = [0u8; 32]; // more than enough for empty payload + CRC
    // uart.read_exact_timeout(&mut rest[..len + CRC_LEN], timeout_ms)?;
    uart.read(&mut rest[..len + CRC_LEN])?;

    // validate CRC
    let mut full = [0u8; HEADER_SIZE + 32];
    full[..HEADER_SIZE].copy_from_slice(&hdr);
    full[HEADER_SIZE..HEADER_SIZE + len + CRC_LEN].copy_from_slice(&rest[..len + CRC_LEN]);

    let rx_crc = u16::from_le_bytes(rest[len..len + 2].try_into().unwrap());
    if calc_crc(&full[..HEADER_SIZE + len]) != rx_crc {
        return Err(EspError::CrcMismatch);
    }

    // validate that it is indeed a PingResp
    if hdr[4] != Module::Ctrl as u8 || hdr[5] != Command::PingResp as u8 {
        return Err(EspError::UnexpectedResponse(hdr[5]));
    }

    Ok(())
}

// todo: BLE scan (get_ble) is identical – just swap Module::Ble, BleScanStart, BleScanResult, and the payload layout (address + adv-data).
pub fn get_aps(uart: &mut Uart, timeout_ms: u32) -> Result<Vec<ApInfo, AP_BUF_MAX>, EspError> {
    let mut out = Vec::<ApInfo, AP_BUF_MAX>::new();

    // 1 → start scan
    let mut tx = [0u8; HEADER_SIZE + CRC_LEN];
    let frame = build_frame(&mut tx, Module::Wifi, Command::WifiScanStart, &[]);
    uart.write(frame)?;

    // 2 → collect results
    loop {
        // read header
        let mut hdr = [0u8; HEADER_SIZE];
        // uart.read_exact_timeout(&mut hdr, timeout_ms)?;
        uart.read(&mut hdr)?;

        let len = u16::from_le_bytes([hdr[2], hdr[3]]) as usize;

        // filter out possible Ctrl ACKs
        if hdr[4] == Module::Ctrl as u8 {
            // todo: Sloppy to discard byte swith static len buf.
            let mut temp_buf = [0; 100];
            uart.read(&mut temp_buf[..len + CRC_LEN])?; // helper that reads & drops
            continue;
        }

        // sanity: expect WifiScanResult
        if hdr[4] != Module::Wifi as u8 || hdr[5] != Command::WifiScanResult as u8 {
            return Err(EspError::UnexpectedResponse(hdr[5]));
        }

        // read payload + CRC
        let mut buf = [0u8; 256];
        // uart.read_exact_timeout(&mut buf[..len + CRC_LEN], timeout_ms)?;
        uart.read(&mut buf[..len + CRC_LEN])?;

        // verify CRC
        let mut full = [0u8; HEADER_SIZE + 256];
        full[..HEADER_SIZE].copy_from_slice(&hdr);
        full[HEADER_SIZE..HEADER_SIZE + len + CRC_LEN].copy_from_slice(&buf[..len + CRC_LEN]);
        let rx_crc = u16::from_le_bytes(buf[len..len + 2].try_into().unwrap());
        if calc_crc(&full[..HEADER_SIZE + len]) != rx_crc {
            return Err(EspError::CrcMismatch);
        }

        // parse payload
        let entries = buf[0] as usize;
        if entries == 0 {
            break;
        } // end-of-scan
        let mut idx = 1;
        for _ in 0..entries {
            let rssi = buf[idx] as i8;
            idx += 1;
            let mut bssid = [0u8; 6];
            bssid.copy_from_slice(&buf[idx..idx + 6]);
            idx += 6;
            let slen = buf[idx] as usize;
            idx += 1;
            let ssid_bytes = &buf[idx..idx + slen];
            idx += slen;

            let mut ssid = String::<32>::new();
            ssid.push_str(core::str::from_utf8(ssid_bytes).unwrap())
                .unwrap();
            out.push(ApInfo { ssid, bssid, rssi }).ok();
        }
    }

    Ok(out)
}

// Aadapted from `esp_hosted_transport.h`.

/* ---------- priority queues & generic constants ---------- */

pub const PRIO_Q_SERIAL: u8 = 0;
pub const PRIO_Q_BT: u8 = 1;
pub const PRIO_Q_OTHERS: u8 = 2;
pub const MAX_PRIORITY_QUEUES: u8 = 3;

pub const MAC_SIZE_BYTES: usize = 6;

/* ---------- serial device ---------- */

pub const SERIAL_IF_FILE: &str = "/dev/esps0";

/* ---------- protobuf / RPC endpoints (same length!) ---------- */

pub const RPC_EP_NAME_RSP: &str = "RPCRsp";
pub const RPC_EP_NAME_EVT: &str = "RPCEvt";

/* ---------- host-side flow-control state ---------- */

#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum FlowCtrl {
    Nc = 0,  // “no change / unknown”
    On = 1,  // host permits ESP to send
    Off = 2, // host asks ESP to pause
}

/* ---------- private packet / event / tag types ---------- */

#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PrivPacketType {
    Event = 0x33,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PrivEventType {
    Init = 0x22,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum SlaveConfigPrivTagType {
    HostCapabilities = 0x44,
    RcvdEspFirmwareChipId = 0x45,
    SlvConfigTestRawTp = 0x46,
    SlvConfigThrottleHighThreshold = 0x47,
    SlvConfigThrottleLowThreshold = 0x48,
}

/* ---------- transport MTU per physical bus ---------- */

pub const ESP_TRANSPORT_SDIO_MAX_BUF_SIZE: usize = 1536;
pub const ESP_TRANSPORT_SPI_MAX_BUF_SIZE: usize = 1600;
pub const ESP_TRANSPORT_SPI_HD_MAX_BUF_SIZE: usize = 1600;
pub const ESP_TRANSPORT_UART_MAX_BUF_SIZE: usize = 1600;

/* ---------- packed event header (flex-array payload follows on the wire) ---------- */

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct PrivEventHeader {
    pub event_type: u8,
    pub event_len: u8, // length of `event_data`
                       // `event_data` bytes follow immediately in the frame
}

/* ---------- checksum helper (identical semantics to C version) ---------- */

/// Sum-of-bytes, wraps at `u16`.
#[inline(always)]
pub fn compute_checksum(buf: &[u8]) -> u16 {
    buf.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as u16))
}

// Adapted from `esp_hosted_bitmasks.h`
// todo: Make into proper enums
// Bit-position and mask definitions that mirror
// `esp_hosted_bitmasks.h` (May 2025).
//
// • pure **`#![no_std]`**, no external crates
// • constants are `const` so the compiler can fold everything away
// • helpers are `#[inline(always)]` zero-cost

/* ------------------------------------------------------------------------- */
/*  small helpers – replacements for the C SET/GET “macros”                  */
/* ------------------------------------------------------------------------- */

/// Returns `val | (1 << pos)`
#[inline(always)]
pub const fn set_bit_u32(val: u32, pos: u8) -> u32 {
    val | (1u32 << pos)
}

/// Returns `((val >> pos) & 1) != 0`
#[inline(always)]
pub const fn get_bit_u32(val: u32, pos: u8) -> bool {
    ((val >> pos) & 1) != 0
}

/* ------------------------------------------------------------------------- */
/*  Wi-Fi **scan-AP-record** flag word                                       */
/* ------------------------------------------------------------------------- */

pub mod wifi_scan_ap_rec {
    /* bit positions (u16) */
    pub const phy_11b: u8 = 0;
    pub const phy_11g: u8 = 1;
    pub const phy_11n: u8 = 2;
    pub const phy_lr: u8 = 3;
    pub const phy_11ax: u8 = 4;
    pub const wps: u8 = 5;
    pub const ftm_responder: u8 = 6;
    pub const ftm_initiator: u8 = 7;
    pub const phy_11a: u8 = 8;
    pub const phy_11ac: u8 = 9;

    /// highest “used” bit index
    pub const MAX_USED_BIT: u8 = 10;

    /* masks --------------------------------------------------------------- */

    /// `0b1111_1100_0000_0000`
    pub const RESERVED_BITMASK: u16 = 0xFC00;

    /* helpers ------------------------------------------------------------- */

    /// Extract the *reserved* bits (already right-aligned).
    #[inline(always)]
    pub const fn get_reserved(num: u16) -> u16 {
        (num & RESERVED_BITMASK) >> MAX_USED_BIT
    }

    /// Overlay `reserved_in` into `num`.
    #[inline(always)]
    pub const fn set_reserved(num: u16, reserved_in: u16) -> u16 {
        num | (reserved_in << MAX_USED_BIT)
    }
}

/* ------------------------------------------------------------------------- */
/*  Wi-Fi **STA-info** flag word                                             */
/* ------------------------------------------------------------------------- */

pub mod wifi_sta_info {
    pub const phy_11b: u8 = 0;
    pub const phy_11g: u8 = 1;
    pub const phy_11n: u8 = 2;
    pub const phy_lr: u8 = 3;
    pub const phy_11ax: u8 = 4;
    pub const is_mesh_child: u8 = 5;

    pub const MAX_USED_BIT: u8 = 6;
    pub const RESERVED_BITMASK: u16 = 0xFFC0;

    #[inline(always)]
    pub const fn get_reserved(num: u16) -> u16 {
        (num & RESERVED_BITMASK) >> MAX_USED_BIT
    }
    #[inline(always)]
    pub const fn set_reserved(num: u16, reserved_in: u16) -> u16 {
        num | (reserved_in << MAX_USED_BIT)
    }
}

/* ------------------------------------------------------------------------- */
/*  HE-AP-info flag word                                                     */
/* ------------------------------------------------------------------------- */

pub mod wifi_he_ap_info {
    /* bits 0-5  →  six-bit BSS-color field (see constant below) */
    pub const partial_bss_color: u8 = 6;
    pub const bss_color_disabled: u8 = 7;

    pub const MAX_USED_BIT: u8 = 8;

    /// `0b0011_1111`
    pub const BSS_COLOR_BITS: u8 = 0x3F;
}

/* ------------------------------------------------------------------------- */
/*  STA-config – **bitfield 1**                                              */
/* ------------------------------------------------------------------------- */

pub mod wifi_sta_config_1 {
    pub const rm_enabled: u8 = 0;
    pub const btm_enabled: u8 = 1;
    pub const mbo_enabled: u8 = 2;
    pub const ft_enabled: u8 = 3;
    pub const owe_enabled: u8 = 4;
    pub const transition_disable: u8 = 5;

    pub const MAX_USED_BIT: u8 = 6;
    pub const RESERVED_BITMASK: u32 = 0xFFFF_FFC0;

    #[inline(always)]
    pub const fn get_reserved(num: u32) -> u32 {
        (num & RESERVED_BITMASK) >> MAX_USED_BIT
    }
    #[inline(always)]
    pub const fn set_reserved(num: u32, reserved_in: u32) -> u32 {
        num | (reserved_in << MAX_USED_BIT)
    }
}

/* ------------------------------------------------------------------------- */
/*  STA-config – **bitfield 2**                                              */
/* ------------------------------------------------------------------------- */
/* Espressif added more bits in IDF v5.5; we reflect that with a Cargo
feature flag `idf_5_5_or_newer`.  */

// pub mod wifi_sta_config_2 {
//     pub const he_dcm_set:                                 u8 = 0;
//     /* multi-bit fields: see constants below */
//     pub const he_dcm_max_constellation_tx_bits:           u8 = 1; /* 2 bits */
//     pub const he_dcm_max_constellation_rx_bits:           u8 = 3; /* 2 bits */
//     pub const he_mcs9_enabled:                            u8 = 5;
//     pub const he_su_beamformee_disabled:                  u8 = 6;
//     pub const he_trig_su_bmforming_feedback_disabled:     u8 = 7;
//     pub const he_trig_mu_bmforming_partial_feedback_disabled: u8 = 8;
//     pub const he_trig_cqi_feedback_disabled:              u8 = 9;
//
//     pub const MAX_USED_BIT:                               u8 = 10;
//     pub const RESERVED_BITMASK:                           u32 = 0xFFFF_FC00;
//
//     #[inline(always)]
//     pub const fn get_reserved(num: u32) -> u32 {
//         (num & RESERVED_BITMASK) >> MAX_USED_BIT
//     }
//     #[inline(always)]
//     pub const fn set_reserved(num: u32, reserved_in: u32) -> u32 {
//         num | (reserved_in << MAX_USED_BIT)
//     }
// }

pub mod wifi_sta_config_2 {
    pub const he_dcm_set: u8 = 0;
    pub const he_dcm_max_constellation_tx_bits: u8 = 1; /* 2 bits */
    pub const he_dcm_max_constellation_rx_bits: u8 = 3; /* 2 bits */
    pub const he_mcs9_enabled: u8 = 5;
    pub const he_su_beamformee_disabled: u8 = 6;
    pub const he_trig_su_bmforming_feedback_disabled: u8 = 7;
    pub const he_trig_mu_bmforming_partial_feedback_disabled: u8 = 8;
    pub const he_trig_cqi_feedback_disabled: u8 = 9;
    pub const vht_su_beamformee_disabled: u8 = 10;
    pub const vht_mu_beamformee_disabled: u8 = 11;
    pub const vht_mcs8_enabled: u8 = 12;

    pub const MAX_USED_BIT: u8 = 13;
    pub const RESERVED_BITMASK: u32 = 0xFFFF_E000;

    #[inline(always)]
    pub const fn get_reserved(num: u32) -> u32 {
        (num & RESERVED_BITMASK) >> MAX_USED_BIT
    }
    #[inline(always)]
    pub const fn set_reserved(num: u32, reserved_in: u32) -> u32 {
        num | (reserved_in << MAX_USED_BIT)
    }
}

/* ------------------------------------------------------------------------- */
/*  Multi-bit-field helper masks                                             */
/* ------------------------------------------------------------------------- */

/* (kept as u8/u32 so callers can OR-in at compile-time) */
pub const WIFI_STA_CONFIG_2_HE_DCM_MAX_CONSTELLATION_TX_MASK: u32 = 0b11 << 1;
pub const WIFI_STA_CONFIG_2_HE_DCM_MAX_CONSTELLATION_RX_MASK: u32 = 0b11 << 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive)]
#[repr(u16)] // todo: Not sure on this one.
enum RpcError {
    RPC_ERR_BASE = ESP_ERR_HOSTED_BASE,
    RPC_ERR_NOT_CONNECTED,
    RPC_ERR_NO_AP_FOUND,
    RPC_ERR_INVALID_PASSWORD,
    RPC_ERR_INVALID_ARGUMENT,
    RPC_ERR_OUT_OF_RANGE,
    RPC_ERR_MEMORY_FAILURE,
    RPC_ERR_UNSUPPORTED_MSG,
    RPC_ERR_INCORRECT_ARG,
    RPC_ERR_PROTOBUF_ENCODE,
    RPC_ERR_PROTOBUF_DECODE,
    RPC_ERR_SET_ASYNC_CB,
    RPC_ERR_TRANSPORT_SEND,
    RPC_ERR_REQUEST_TIMEOUT,
    RPC_ERR_REQ_IN_PROG,
    RPC_ERR_SET_SYNC_SEM,
}
