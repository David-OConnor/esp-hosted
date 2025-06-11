#![no_std]
#![allow(dead_code)]
#![allow(non_camel_case_types)]

//! An interface for interacting with ESP-Hosted-MCU firmware, via UART.

mod protocol;
mod rf;
mod transport;
mod util;
// mod misc;

use core::sync::atomic::{AtomicBool, Ordering};

use defmt::println;
#[cfg(feature = "hal")]
use hal::{
    pac::{SPI2, USART1, USART2},
    spi::{Spi, SpiError},
    usart::{UartError, Usart},
};
use heapless::{String, Vec};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[macro_export]
macro_rules! parse_le {
    ($bytes:expr, $t:ty, $range:expr) => {{ <$t>::from_le_bytes($bytes[$range].try_into().unwrap()) }};
}

#[macro_export]
macro_rules! copy_le {
    ($dest:expr, $src:expr, $range:expr) => {{ $dest[$range].copy_from_slice(&$src.to_le_bytes()) }};
}

use crate::protocol::{Module, build_frame, HEADER_SIZE, RpcId, CRC_SIZE, RpcHeader, RPC_HEADER_MAX_SIZE};
use crate::transport::compute_checksum;

#[cfg(feature = "hal")]
// todo: Allow any uart.
type Uart = Usart<USART2>;
// type Uart = Usart<USART1>;

// todo: How can we make this flexible? EH?

const AP_BUF_MAX: usize = 100;
const BLE_BUF_MAX: usize = 100;

const ESP_ERR_HOSTED_BASE: u16 = 0x2f00;

const MORE_FRAGMENT: u8 = 1 << 0; // todo type and if we use it.

/// A simple error enum for our host-side protocol
#[derive(Debug)]
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

#[cfg(feature = "hal")]
/// Round-trip health-check.  Returns Err on timeout / CRC / protocol error.
pub fn status_check(uart: &mut Uart, timeout_ms: u32) -> Result<(), EspError> {
    const FRAME_LEN: usize = HEADER_SIZE + RPC_HEADER_MAX_SIZE + 9 + CRC_SIZE;

    let mut frame_buf = [0; FRAME_LEN];

    let iface_num = 0;  // 0 = STA, 1 = AP

    // todo: Put this in a build-RPC fn, or build it into build_frame.
    let rpc_header = RpcHeader {
        id: RpcId::ReqWifiApGetStaList,
        len: 1, // Payload len of 1: Interface number.
    };

    let mut payload = [0; 9];

    // todo: Is it always 5? If so, use RPC_HEADER_SIZE = 5.
    let rpc_header_size = rpc_header.to_bytes(&mut payload);
    let mut i =  rpc_header_size;
    println!("RPC HEADER SIZE: {:?}", rpc_header_size);

    // field #2, wire-type=2 ?
    // copy_le!(payload, 12_u16, i..i + 2);
    // todo: Look up where this comes from.
    payload[i] = 0x12;  // Tag byte.
    i += 1;

    // This is a variant.
    payload[i] = 1; // Body length in bytes?
    i += 1;

    payload[i] = iface_num;
    i += 1;

    build_frame(&mut frame_buf, &payload);

    // todo: Do we copy the payload checksum at bytes 6..8 to the end of the buffer?

    uart.write(&frame_buf)?;

    // todo: Experimenting with slip_buf. wrap this in a helper if required.
    // let mut slip_buf = [0u8; 2 * PING_FRAME_LEN + 2]; // worst-case expansion
    // let slip_len = slip_encode(&frame_buf[..frame_len], &mut slip_buf);
    // uart.write(&slip_buf[..slip_len])?;

    println!("Writing status check frame: {:?}", &frame_buf);

    // let mut hdr = [0; HEADER_SIZE];
    let mut hdr = [0; 12];

    // uart.read_exact_timeout(&mut hdr, timeout_ms)?;
    uart.read(&mut hdr)?;

    println!("Header buf read: {:?}", hdr);

    // if hdr[0] != MAGIC || hdr[1] != VERSION {
    //     println!("ESP Unexpected magic or version"); // todo temp
    //     return Err(EspError::UnexpectedResponse(hdr[0]));
    // }
    let len = u16::from_le_bytes([hdr[2], hdr[3]]) as usize;

    return Ok(());

    // --------- receive payload + CRC ---------
    let mut rest = [0; 1_026]; // more than enough for empty payload + CRC
    // uart.read_exact_timeout(&mut rest[..len + CRC_LEN], timeout_ms)?;
    uart.read(&mut rest[..len])?;

    // validate CRC
    let mut full = [0u8; HEADER_SIZE + 1_026];
    full[..HEADER_SIZE].copy_from_slice(&hdr);
    full[HEADER_SIZE..HEADER_SIZE + len].copy_from_slice(&rest[..len]);

    let rx_crc = u16::from_le_bytes(rest[len..len].try_into().unwrap());
    if compute_checksum(&full[..HEADER_SIZE + len]) != rx_crc {
        println!("ESP CRC mismatch"); // todo temp
        return Err(EspError::CrcMismatch);
    }

    // // validate that it is indeed a PingResp
    // if hdr[4] != Module::Ctrl as u8 || hdr[5] != Command::PingResp as u8 {
    //     println!("ESP Unexpected resp"); // todo temp
    //     return Err(EspError::UnexpectedResponse(hdr[5]));
    // }

    Ok(())
}
