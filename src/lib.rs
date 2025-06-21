#![no_std]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(static_mut_refs)]

//! An interface for interacting with ESP-Hosted-MCU firmware, via UART.

mod header;
pub mod proto_data;
pub mod rf;
mod rpc;
mod transport;

mod esp_hosted_proto;
mod esp_errors;

use micropb::{MessageDecode, MessageEncode, PbDecoder, PbEncoder};

use defmt::{Format, println};

pub use header::PayloadHeader;
use num_enum::TryFromPrimitive;
pub use proto_data::*;
pub use rpc::InterfaceType;
pub use rf::*;

pub use esp_hosted_proto::Rpc as RpcP;
pub use esp_hosted_proto::RpcId as RpcIdP;
pub use esp_hosted_proto::RpcType as RpcTypeP;
pub use esp_hosted_proto::*;


pub use crate::header::HEADER_SIZE;
use crate::{
    header::{PL_HEADER_SIZE},
    proto_data::RpcId,
    rpc::{RPC_MIN_SIZE, Rpc, setup_rpc},
};
// use crate::esp_errors::EspCode;

pub use crate::rpc::*;

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

const MORE_FRAGMENT: u8 = 1 << 0; // todo type and if we use it.

const FRAME_LEN_TX: usize = HEADER_SIZE + RPC_MIN_SIZE + 300; // todo: A/R
// const DATA_LEN_TX: usize = 200; // todo: A/R

static mut TX_BUF: [u8; FRAME_LEN_TX] = [0; FRAME_LEN_TX];
// static mut DATA_BUF: [u8; DATA_LEN_TX] = [0; DATA_LEN_TX];

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
    // todo: Put back. flash limit problem.
    // Esp(EspCode),
}

// #[cfg(feature = "hal")]
// impl From<UartError> for EspError {
//     fn from(e: UartError) -> Self {
//         EspError::Uart(e)
//     }
// }


pub fn cfg_heartbeat<W>(mut write: W, uid: u32, cfg: &RpcReqConfigHeartbeat) -> Result<(), EspError>
// todo: Typedef this if able. (Unstable feature)
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqConfigHeartbeat, uid);

    let mut data = [0; 5]; // Seems to be 4 in for small duration values.
    let data_size = cfg.to_bytes(&mut data);

    unsafe {
        let frame_len = setup_rpc(&mut TX_BUF, &rpc, &data[..data_size]);
        write(&TX_BUF[..frame_len])?;
    }

    Ok(())
}

pub struct ParsedMsg<'a> {
    pub header: PayloadHeader,
    pub rpc: Rpc,
    pub data: &'a[u8],
    pub rpc_proto: RpcP,
}

/// Parse the payload header, and separate the RPC bytes from the whole message. Accepts
/// the whole message received.
// pub fn parse_msg(buf: &[u8]) -> Result<(PayloadHeader, Rpc, &[u8], RpcP), EspError> {
pub fn parse_msg(buf: &[u8]) -> Result<ParsedMsg, EspError> {
    let header = PayloadHeader::from_bytes(&buf[..HEADER_SIZE])?;
    let total_size = header.len as usize + PL_HEADER_SIZE;

    if total_size > buf.len() {
        return Err(EspError::InvalidData);
    }

    let rpc_buf = &buf[HEADER_SIZE..total_size];

    let (rpc, data_start_i, data_len_rpc) = Rpc::from_bytes(rpc_buf)?;

    // Parsing the proto data directly.
    let mut decoder = PbDecoder::new(rpc_buf);

    let mut rpc_proto = RpcP::default();
    rpc_proto.decode(&mut decoder, rpc_buf.len()).map_err(|_| EspError::InvalidData)?;

    let data = &rpc_buf[data_start_i..];

    if data.len() == 2 {
        // todo: Ideally we return an error, but I'm not sure how to confirm that this
        // todo value truly is just for errors.
        let err = decode_varint(data)?.0 as u16;

        // todo: Flash limit problem.
        // if let Ok(e) = TryInto::<EspCode>::try_into(err) {
        // println!("Esp error: {:?}", e);
    }

    Ok(ParsedMsg { header, rpc, data, rpc_proto    })
}
