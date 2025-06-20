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

use micropb::{MessageDecode, MessageEncode, PbDecoder, PbEncoder};

use defmt::{Format, println};
#[cfg(feature = "hal")]
use hal::{
    pac::USART2,
    usart::{UartError, Usart},
};
pub use header::PayloadHeader;
use num_enum::TryFromPrimitive;
pub use proto_data::*;
pub use rf::*;

pub use esp_hosted_proto::Rpc as RpcP;
pub use esp_hosted_proto::*;

pub use crate::header::HEADER_SIZE;
use crate::{
    header::{PL_HEADER_SIZE},
    proto_data::RpcId,
    rpc::{RPC_MIN_SIZE, Rpc, setup_rpc},
};

#[macro_export]
macro_rules! parse_le {
    ($bytes:expr, $t:ty, $range:expr) => {{ <$t>::from_le_bytes($bytes[$range].try_into().unwrap()) }};
}

#[macro_export]
macro_rules! copy_le {
    ($dest:expr, $src:expr, $range:expr) => {{ $dest[$range].copy_from_slice(&$src.to_le_bytes()) }};
}

#[cfg(feature = "hal")]
// todo: Allow any uart.
type Uart = Usart<USART2>;

// todo: How can we make this flexible? EH?

const AP_BUF_MAX: usize = 100;
const BLE_BUF_MAX: usize = 100;

const ESP_ERR_HOSTED_BASE: u16 = 0x2f00;

const MORE_FRAGMENT: u8 = 1 << 0; // todo type and if we use it.

const FRAME_LEN_TX: usize = HEADER_SIZE + RPC_MIN_SIZE + 300; // todo: A/R
const DATA_LEN_TX: usize = 200; // todo: A/R

static mut TX_BUF: [u8; FRAME_LEN_TX] = [0; FRAME_LEN_TX];
static mut DATA_BUF: [u8; DATA_LEN_TX] = [0; DATA_LEN_TX];

/// A simple error enum for our host-side protocol
#[derive(Debug, Format)]
pub enum EspError {
    #[cfg(feature = "hal")]
    Uart(UartError),
    /// e.g. uart, spi etc.
    Comms,
    UnexpectedResponse(u8),
    CrcMismatch,
    Timeout,
    InvalidData,
    // todo: etc. as needed
}

#[cfg(feature = "hal")]
impl From<UartError> for EspError {
    fn from(e: UartError) -> Self {
        EspError::Uart(e)
    }
}

/// Returns size written.
pub fn cfg_heartbeat_pl(buf: &mut [u8], uid: u32, cfg: &RpcReqConfigHeartbeat) -> usize {
    let rpc = Rpc::new_req(RpcId::ReqConfigHeartbeat, 0);

    let mut data = [0; 6]; // Seems to be 4 in for small duration values.
    let data_len = cfg.to_bytes(&mut data);

    let frame_len = setup_rpc(buf, &rpc, &data[..data_len]);

    frame_len
}

pub fn cfg_heartbeat<W>(mut write: W, cfg: &RpcReqConfigHeartbeat, uid: u32) -> Result<(), EspError>
// todo: Typedef this if able.
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    unsafe {
        let frame_len = cfg_heartbeat_pl(&mut TX_BUF, uid, cfg);
        write(&TX_BUF[..frame_len])?;
    }

    Ok(())
}

/// Parse the payload header, and separate the RPC bytes from the whole message. Accepts
/// the whole message received.
pub fn parse_msg(buf: &[u8]) -> Result<(PayloadHeader, Rpc, &[u8], RpcP), EspError> {
    let header = PayloadHeader::from_bytes(&buf[..HEADER_SIZE]);
    let total_size = header.len as usize + PL_HEADER_SIZE;

    if total_size > buf.len() {
        return Err(EspError::InvalidData);
    }

    let rpc_buf = &buf[HEADER_SIZE..total_size];

    // println!("RPC BUF rx: {:?}", rpc_buf);

    let (rpc, data_start_i, data_len_rpc) = Rpc::from_bytes(rpc_buf)?;

    // Parsing the proto data directly.
    let mut decoder = PbDecoder::new(rpc_buf);

    let mut rpc_proto = RpcP::default();
    rpc_proto.decode(&mut decoder, rpc_buf.len()).map_err(|_| EspError::InvalidData)?;

    let data_buf = &rpc_buf[data_start_i..];

    Ok((header, rpc, &data_buf, rpc_proto))
}
