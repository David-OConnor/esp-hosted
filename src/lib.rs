#![no_std]
#![allow(dead_code)]
#![allow(non_camel_case_types)]

//! An interface for interacting with ESP-Hosted-MCU firmware, via UART.

mod protocol;
mod rf;
mod transport;
// mod misc;

use core::sync::atomic::{AtomicBool, Ordering};

use crc_any::{CRCu8, CRCu16};
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

use crate::protocol::{CRC_LEN,  Module, PL_HEADER_SIZE, TLV_HEADER_SIZE,  build_frame, slip_encode, TlvType};

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
    const PING_FRAME_LEN: usize = PL_HEADER_SIZE + TLV_HEADER_SIZE + CRC_LEN;

    let mut frame_buf = [0u8; PING_FRAME_LEN];

    let endpoint = [0]; // todo temp??
    build_frame(&mut frame_buf, TlvType::Data, TlvType::Data, &endpoint, &[]);
    // build_frame(&mut frame_buf, Module::Ctrl, Command::PingReq, &[]);

    let frame_len = PL_HEADER_SIZE + TLV_HEADER_SIZE + 0 + CRC_LEN;

    uart.write(&frame_buf)?;

    // todo: Experimenting with slip_buf. wrap this in a helper if required.
    // let mut slip_buf = [0u8; 2 * PING_FRAME_LEN + 2]; // worst-case expansion
    // let slip_len = slip_encode(&frame_buf[..frame_len], &mut slip_buf);
    // uart.write(&slip_buf[..slip_len])?;

    println!("Writing status check frame: {:?}", &frame_buf);

    // let mut hdr = [0; TLV_HEADER_SIZE];
    let mut hdr = [0; PL_HEADER_SIZE];

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
    uart.read(&mut rest[..len + CRC_LEN])?;

    // validate CRC
    let mut full = [0u8; TLV_HEADER_SIZE + 1_026];
    full[..TLV_HEADER_SIZE].copy_from_slice(&hdr);
    full[TLV_HEADER_SIZE..TLV_HEADER_SIZE + len].copy_from_slice(&rest[..len]);

    let rx_crc = u16::from_le_bytes(rest[len..len + CRC_LEN].try_into().unwrap());
    if compute_checksum(&full[..TLV_HEADER_SIZE + len]) != rx_crc {
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
