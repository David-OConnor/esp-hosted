//! This module contains Wi-Fi and BLE-specific functionality.

use core::default;

use defmt::{Format, println};
use hal::usart::UartError;
use heapless::{String, Vec};

#[cfg(feature = "hal")]
use crate::Uart;
use crate::{
    AP_BUF_MAX, EspError, RpcReqWifiInit, TX_BUF, WifiInitConfig,
    header::HEADER_SIZE,
    proto_data::RpcId,
    rpc::{RPC_MIN_SIZE, Rpc, setup_rpc},
};

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

#[cfg(feature = "hal")]
// todo: BLE scan (get_ble) is identical – just swap Module::Ble, BleScanStart, BleScanResult, and the payload layout (address + adv-data).
pub fn get_aps(uart: &mut Uart, timeout_ms: u32) -> Result<Vec<ApInfo, { AP_BUF_MAX }>, EspError> {
    const FRAME_LEN_TX: usize = HEADER_SIZE + RPC_MIN_SIZE + 5;
    let mut frame_buf = [0; FRAME_LEN_TX];

    let iface_num = 0; // todo: or 1?
    let data = [iface_num];
    // let data = [];

    let rpc = Rpc::new_req(RpcId::ReqWifiApGetStaList, 0);

    let frame_len = setup_rpc(&mut frame_buf, &rpc, &data);
    println!("Total frame size sent: {:?}", frame_len);
    println!("Writing frame: {:?}", &frame_buf[..frame_len]);

    uart.write(&frame_buf[..frame_len])?;

    // let mut hdr = [0; HEADER_SIZE];
    // let mut hdr = [0; 12];
    let mut hdr = [0; 4];

    // uart.read_exact_timeout(&mut hdr, timeout_ms)?;
    uart.read(&mut hdr)?;

    println!("Header buf read: {:?}", hdr);

    let len = u16::from_le_bytes([hdr[2], hdr[3]]) as usize;

    Ok(Vec::new())
}

#[derive(Clone, Copy, Format)]
pub enum WifiMode {
    A,
}

/// Returns size written.
pub fn get_wifi_mode_pl(buf: &mut [u8], uid: u32) -> usize {
    let data = [];

    let data_len = 0;
    let rpc = Rpc::new_req(RpcId::ReqGetWifiMode, 0);

    let frame_len = setup_rpc(buf, &rpc, &data[0..data_len]);

    println!("Writing frame: {:?}", &buf[..frame_len]);

    frame_len
}

#[cfg(feature = "hal")]
pub fn get_wifi_mode(uart: &mut Uart) -> Result<WifiMode, EspError> {
    unsafe {
        let frame_len = get_wifi_mode_pl(&mut TX_BUF, 0);
        uart.write(&TX_BUF[..frame_len])?;
    }

    Ok(WifiMode::A)
}

// todo: Macros may help.

/// Returns size written.
pub fn wifi_init_pl(buf: &mut [u8], uid: u32, cfg: &WifiInitConfig) -> usize {
    let rpc = Rpc::new_req(RpcId::ReqWifiInit, 0);

    // todo: A/R.
    let mut data = [0; 200];

    let pl = RpcReqWifiInit {
        cfg: cfg.clone(), // todo: Don't clone...
    };

    let data_len = pl.to_bytes(&mut data);

    let frame_len = setup_rpc(buf, &rpc, &data[..data_len]);

    println!("Wifi init req data size: {:?}", data_len);
    println!("Writing frame: {:?}", &buf[..frame_len]);

    frame_len
}

// #[cfg(feature = "hal")]
// pub fn wifi_init(uart: &mut Uart) -> Result<i32, EspError> {
pub fn wifi_init<W>(mut write: W) -> Result<i32, EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    // todo: Pass as a param A/R.

    let cfg = WifiInitConfig {
        // todo: Set up fields.
        ..Default::default()
    };

    unsafe {
        let frame_len = wifi_init_pl(&mut TX_BUF, 0, &cfg);
        // uart.write(&TX_BUF[..frame_len])?;
        write(&TX_BUF[..frame_len])?;
    }

    Ok(0)
}
