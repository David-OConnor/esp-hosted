#![no_std]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(static_mut_refs)]

//! # ESP Hosted
//! For connecting to an [ESP-Hosted-MCU](https://!github.com/espressif/esp-hosted-mcu) from a Host MCU with firmware
//! written in rust.
//!
//! Compatible with ESP-Hosted-MCU 2.0.6 and ESP IDF 5.4.1 (And likely anything newer), and any host MCU and architecture.
//! For details on ESP-HOSTED-MCU's protocol see
//! [this document](/esp_hosted_protocol.md). For how to use the commands in the library effectively, reference the
//! [ESP32 IDF API docs](https://!docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html)
//!
//! This library includes two approaches: A high-level API using data structures from this library, and full access to
//! the native protobuf structures. The native API is easier to work with, but only implements a small portion of functionality.
//! The protobuf API is complete, but more cumbersome.
//!
//! This library does not use an allocator. This makes integrating it simple, but it uses a significant amount of flash
//! for static buffers. These are configured in the `build_proto/src/main.rs` script on a field-by-field basis.
//!
//! It's transport agnostic; compatible with SPI, SDIO, and UART. It does this by allowing the application firmware to pass
//! a generic `write` function, and reads are performed as functions that act on buffers passed by the firmware.

mod header;
pub mod proto_data;
mod rpc;
mod transport;
pub mod wifi;

pub mod ble;
mod esp_errors;
pub mod proto;
mod util;

// pub use ble::*;
use defmt::{Format, println};
pub use esp_errors::EspCode;
pub use header::{PayloadHeader, build_frame_ble};
use micropb::{MessageDecode, PbDecoder};
pub use proto::{Rpc as RpcP, RpcId as RpcIdP, RpcType as RpcTypeP};
pub use proto_data::RpcId;

pub use crate::rpc::*;
use crate::{
    header::{HEADER_SIZE, InterfaceType, PL_HEADER_SIZE},
    proto_data::RpcReqConfigHeartbeat,
    transport::PacketType,
};

#[macro_export]
macro_rules! parse_le {
    ($bytes:expr, $t:ty, $range:expr) => {{ <$t>::from_le_bytes($bytes[$range].try_into().unwrap()) }};
}

#[macro_export]
macro_rules! copy_le {
    ($dest:expr, $src:expr, $range:expr) => {{ $dest[$range].copy_from_slice(&$src.to_le_bytes()) }};
}

const AP_BUF_MAX: usize = 100;
const BLE_BUF_MAX: usize = 100;

const ESP_ERR_HOSTED_BASE: u16 = 0x2f00;

/// A simple error enum for our host-side protocol
#[derive(Format)]
pub enum EspError {
    // #[cfg(feature = "hal")]
    // Uart(UartError),
    /// e.g. uart, spi etc.
    Comms,
    UnexpectedResponse(u8),
    CrcMismatch,
    Timeout,
    InvalidData,
    Proto,
    Capacity,
    // todo: Put back. flash limit problem.
    Esp(EspCode),
}

// #[cfg(feature = "hal")]
// impl From<UartError> for EspError {
//     fn from(e: UartError) -> Self {
//         EspError::Uart(e)
//     }
// }

/// Minimum of 10s.
pub fn cfg_heartbeat<W>(
    buf: &mut [u8],
    mut write: W,
    uid: u32,
    cfg: &RpcReqConfigHeartbeat,
) -> Result<(), EspError>
// todo: Typedef this if able. (Unstable feature)
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqConfigHeartbeat, uid);

    let mut data = [0; 5]; // Seems to be 4 in for small duration values.
    let data_size = cfg.to_bytes(&mut data);

    // unsafe {
    let frame_len = setup_rpc(buf, &rpc, &data[..data_size]);
    write(&buf[..frame_len])?;
    // }

    Ok(())
}

pub struct WifiMsg<'a> {
    pub header: PayloadHeader,
    pub rpc: Rpc,
    pub data: &'a [u8],
    // pub rpc_raw: Option<RpcP>,
    /// This is a result, because sometimes it can fail, e.g. due to a capacity error,
    /// where we're able to parse the rest of the data directly.
    pub rpc_parsed: Result<RpcP, EspError>,
}

pub struct HciMsg<'a> {
    pub data: &'a [u8],
}

pub enum MsgParsed<'a> {
    Wifi(WifiMsg<'a>),
    Hci(HciMsg<'a>),
}

/// Parse the payload header, and separate the RPC bytes from the whole message. Accepts
/// the whole message received.
pub fn parse_msg(buf: &[u8]) -> Result<MsgParsed, EspError> {
    // Check for a shifted packet due to jitter. For example, from late reception start.
    // This will cut of information that may be important for Wi-Fi RPC packets, but is skippable
    // for HCI.

    if buf[0] > 8 || buf[0] == 0 {
        // Handle a small amount of jitter (delayed reception) for BLE packets.
        const MAX_SHIFT: usize = 6; // Index of offset len.

        // Note: This approach starts by looking at byte 4, then moves the starting index left
        // each index, effectively. It covers both forward, and reverse shifts by a few bytes.
        for offset in 1..MAX_SHIFT {
            if buf[offset..offset + 2] == [12, 0] && buf[offset + 6..offset + 9] == [0, 0, 4] {
                // Align the shift with the [12, 0] we matched.
                let shift = 4 - offset;

                return Ok(MsgParsed::Hci(HciMsg {
                    data: &buf[PL_HEADER_SIZE - shift..],
                }));
            }
        }

        // Check for more aggressive shifts as well, without relying on the [12, 0] offset,
        // and assuming HCI packet type = 62
        for offset in 1..16 {
            // println!("Checking for match T2: {:?}", buf[offset..offset + 9]);
            // if buf[offset..offset + 4] == [0, 0, 4, 62] {
            if buf[offset..offset + 3] == [0, 4, 62] {
                // Align the shift with the [12, 0] we matched.
                let shift = 9 - offset;
                // println!(
                //     "Shifted BLE packet Type 2. Shift: {}. Offset: {}",
                //     shift, offset
                // );

                // println!("Corrected buf T2: {:?}",&buf[PL_HEADER_SIZE - shift..30]); // todo tmep
                return Ok(MsgParsed::Hci(HciMsg {
                    data: &buf[PL_HEADER_SIZE - shift..],
                }));
            }
        }

        // todo: Handle shifted Wi-Fi packets too?
        // let mut modded_header_buf = [0; PL_HEADER_SIZE];
        //
        // modded_header_buf[shift..].copy_from_slice(&buf[shift..PL_HEADER_SIZE - shift]);
        // println!("Modded header: {:?}", modded_header_buf);
        //
        // header = PayloadHeader::from_bytes(&mut modded_header_buf)?;
        // header.if_type = InterfaceType::Hci;
    }

    let mut header = PayloadHeader::from_bytes(&buf[..HEADER_SIZE])?;
    let mut total_size = header.len as usize + PL_HEADER_SIZE;

    if total_size > buf.len() {
        // todo: Print is temp.
        println!(
            "\nMsg size exceeds buf len. Size: {}, buf len: {}",
            total_size,
            buf.len()
        );
        return Err(EspError::Capacity);
    }

    if header.if_type == InterfaceType::Hci {
        return Ok(MsgParsed::Hci(HciMsg {
            data: &buf[PL_HEADER_SIZE..],
        }));
    }

    if HEADER_SIZE >= total_size {
        // todo: Print is temp.
        println!(
            "Error: Invalid RPC packet. packet size: {}, buf: {:?}",
            total_size,
            buf[0..24]
        );
        return Err(EspError::InvalidData);
    }

    let rpc_buf = &buf[HEADER_SIZE..total_size];
    let (rpc, data_start_i, _data_len_rpc) = Rpc::from_bytes(rpc_buf)?;
    let data = &rpc_buf[data_start_i..];

    // Parsing the proto data from the generated mod.
    // let mut decoder = PbDecoder::new(&rpc_buf[0..100]);
    let mut decoder = PbDecoder::new(rpc_buf);
    let mut rpc_parsed = RpcP::default();

    let rpc_parsed = match rpc_parsed.decode(&mut decoder, rpc_buf.len()) {
        Ok(r) => Ok(rpc_parsed),
        Err(e) => Err(EspError::Proto),
    };
    // rpc_parsed
    //     .decode(&mut decoder, rpc_buf.len())
    //     .map_err(|_| EspError::Proto)?;

    Ok(MsgParsed::Wifi(WifiMsg {
        header,
        rpc,
        data,
        rpc_parsed,
    }))
}
