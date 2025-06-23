#![no_std]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(static_mut_refs)]

//! An interface for interacting with ESP-Hosted-MCU firmware, via UART.

mod header;
pub mod proto_data;
mod rpc;
mod transport;
pub mod wifi;

mod esp_errors;
pub mod proto;
mod util;

use defmt::{Format, println};
pub use header::PayloadHeader;
use micropb::{MessageDecode, MessageEncode, PbDecoder};
use num_enum::TryFromPrimitive;
pub use proto::{Rpc as RpcP, RpcId as RpcIdP, RpcType as RpcTypeP, *};
pub use proto_data::RpcId;

// use crate::esp_errors::EspCode;
pub use crate::rpc::*;
use crate::{
    header::{HEADER_SIZE, PL_HEADER_SIZE},
    proto_data::RpcReqConfigHeartbeat,
    rpc::{Rpc, setup_rpc},
    wifi::WifiApRecord,
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
    // Esp(EspCode),
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

pub struct ParsedMsg<'a> {
    pub header: PayloadHeader,
    pub rpc: Rpc,
    pub data: &'a [u8],
    pub rpc_raw: Option<RpcP>,
}

/// Parse the payload header, and separate the RPC bytes from the whole message. Accepts
/// the whole message received.
pub fn parse_msg(buf: &[u8]) -> Result<ParsedMsg, EspError> {
    let header = PayloadHeader::from_bytes(&buf[..HEADER_SIZE])?;
    let total_size = header.len as usize + PL_HEADER_SIZE;

    if total_size > buf.len() {
        return Err(EspError::Capacity);
    }

    let rpc_buf = &buf[HEADER_SIZE..total_size];
    let (rpc, data_start_i, _data_len_rpc) = Rpc::from_bytes(rpc_buf)?;
    let data = &rpc_buf[data_start_i..];

    // todo: Temp to get troubleshooting data to the micropb GH.
    println!("RPC BUF: {:?}", rpc_buf);
    // println!("\n\n\n\nParsing msg..."); // todo tmep

    // Parsing the proto data from the generated mod.
    let mut decoder = PbDecoder::new(rpc_buf);
    let mut rpc_proto = RpcP::default();

    // todo: Until we sort out decode errors on repeated fields with micropb.
    // rpc_proto
    //     .decode(&mut decoder, rpc_buf.len())
    //     .map_err(|_| EspError::Proto)?;

    // Workaround to fall back to our native API, setting MicroPB's parse result to None, vice returning an error.
    let mut rpc_proto_ = None;
    match rpc_proto.decode(&mut decoder, rpc_buf.len()) {
        Ok(_) => rpc_proto_ = Some(rpc_proto),
        Err(e) => {
            match e {
                micropb::DecodeError::ZeroField => println!("ZF"),
                micropb::DecodeError::UnexpectedEof => println!("Ueof"),
                micropb::DecodeError::Deprecation => println!("Dep"),
                micropb::DecodeError::UnknownWireType => println!("UWT"),
                micropb::DecodeError::VarIntLimit => println!("Varlint"),
                micropb::DecodeError::CustomField => println!("CustomF"),
                micropb::DecodeError::Utf8 => println!("Utf"),
                micropb::DecodeError::Capacity => {
                    // println!("RPC buf len: {} Msg size (micropb): {}", rpc_buf.len(), rpc_proto.compute_size());
                    println!("MIcropb capacity error");
                }
                micropb::DecodeError::WrongLen => println!("WrongLen"),
                micropb::DecodeError::Reader(e2) => println!("Reader"),
                _ => println!("Other"),
            }
            println!("Micropb decode error on: {:?}", rpc.msg_type);
        }
    }

    Ok(ParsedMsg {
        header,
        rpc,
        data,
        rpc_raw: rpc_proto_,
    })
}
