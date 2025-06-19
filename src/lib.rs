#![no_std]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(static_mut_refs)]

//! An interface for interacting with ESP-Hosted-MCU firmware, via UART.

mod protocol;
mod rf;
mod rpc;
mod proto_data;
mod transport;
mod util;
// mod misc;

pub enum DataError {
    Invalid,
}

use defmt::{println, Format};
#[cfg(feature = "hal")]
use hal::{
    pac::{USART2},
    usart::{UartError, Usart},
};
use num_enum::{TryFromPrimitive};
use crate::proto_data::{RpcId, RpcReqConfigHeartbeat};
use crate::protocol::{HEADER_SIZE, RPC_MIN_SIZE};
use crate::rf::{get_wifi_mode, req_wifi_init, wifi_init};
use crate::rpc::{setup_rpc, Rpc};

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
// type Uart = Usart<USART1>;

// todo: How can we make this flexible? EH?

const AP_BUF_MAX: usize = 100;
const BLE_BUF_MAX: usize = 100;

const ESP_ERR_HOSTED_BASE: u16 = 0x2f00;

const MORE_FRAGMENT: u8 = 1 << 0; // todo type and if we use it.

const FRAME_LEN_TX: usize = HEADER_SIZE + RPC_MIN_SIZE + 200; // todo: A/R
static mut TX_BUF: [u8; FRAME_LEN_TX] = [0; FRAME_LEN_TX];

/// A simple error enum for our host-side protocol
#[derive(Debug, Format)]
pub enum EspError {
    #[cfg(feature = "hal")]
    Uart(UartError),
    UnexpectedResponse(u8),
    CrcMismatch,
    Timeout,
    // todo: etc. as needed
}

#[cfg(feature = "hal")]
impl From<UartError> for EspError {
    fn from(e: UartError) -> Self {
        EspError::Uart(e)
    }
}


/// Returns size written.
pub fn config_heartbeat(buf: &mut [u8], uid: u32, cfg: &RpcReqConfigHeartbeat) -> usize {

    let rpc = Rpc::new_req(RpcId::ReqConfigHeartbeat, 0);
    let mut data = [0; 6]; // todo?
    let data_len = cfg.to_bytes(&mut data);

    let frame_len = setup_rpc(buf, &rpc, &data[..data_len]);

    println!("Total frame size sent: {:?}", frame_len);
    println!("Writing frame: {:?}", &buf[..frame_len]);

    frame_len
}


#[cfg(feature = "hal")]
pub fn status_check(uart: &mut Uart) -> Result<(), EspError> {
    // let mode = get_wifi_mode(uart);
    // println!("Wifi mode: {:?}", mode);

    // wifi_init(uart)?;

    let cfg = RpcReqConfigHeartbeat {
        enable: true,
        duration: 1,
    };

    unsafe {
        let frame_len = config_heartbeat(&mut TX_BUF, 0, &cfg);
        uart.write(&TX_BUF[..frame_len])?;
    }

    let mut rx_buf = [0; 6];

    // uart.read_exact_timeout(&mut hdr, timeout_ms)?;
    uart.read(&mut rx_buf)?;

    Err(EspError::Timeout)
}
