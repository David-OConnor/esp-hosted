//! This module contains Wi-Fi and BLE-specific functionality.

use defmt::{println, Format};
use heapless::{String, Vec};

use crate::{EspError, protocol::{HEADER_SIZE}, rpc::{Rpc, setup_rpc}, proto_data::RpcId, AP_BUF_MAX, TX_BUF};
#[cfg(feature = "hal")]
use crate::{Uart};
use crate::protocol::RPC_MIN_SIZE;

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
pub fn get_aps(
    uart: &mut Uart,
    timeout_ms: u32,
) -> Result<Vec<ApInfo, { AP_BUF_MAX }>, EspError> {
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
    A
}

/// Returns size written.
pub fn req_wifi_init(buf: &mut [u8], uid: u32) -> usize {
    let data = [];

    let rpc = Rpc::new_req(RpcId::ReqWifiInit, 0);

    let frame_len = setup_rpc(buf, &rpc, &data);

    println!("Total frame size sent: {:?}", frame_len);
    println!("Writing frame: {:?}", &buf[..frame_len]);

    frame_len
}


/// Returns size written.
pub fn req_get_wifi_mode(buf: &mut [u8], uid: u32) -> usize {
    let data = [];

    let rpc = Rpc::new_req(RpcId::ReqGetWifiMode, 0);

    let frame_len = setup_rpc(buf, &rpc, &data);

    println!("Total frame size sent: {:?}", frame_len);
    println!("Writing frame: {:?}", &buf[..frame_len]);

    frame_len
}


#[cfg(feature = "hal")]
pub fn wifi_init(uart: &mut Uart) -> Result<i32, EspError> {
    unsafe {
        let frame_len = req_wifi_init(&mut TX_BUF, 0);
        uart.write(&TX_BUF[..frame_len])?;
    }

    let mut rx_buf = [0; 6];

    // uart.read_exact_timeout(&mut hdr, timeout_ms)?;
    uart.read(&mut rx_buf)?;

    println!("Rx buf read: {:?}", rx_buf);

    let len = u16::from_le_bytes([rx_buf[2], rx_buf[3]]) as usize;

    Ok(0)
}

#[cfg(feature = "hal")]
pub fn get_wifi_mode(uart: &mut Uart) -> Result<WifiMode, EspError> {
    unsafe {
        let frame_len = req_get_wifi_mode(&mut TX_BUF, 0);
        uart.write(&TX_BUF[..frame_len])?;
    }


    let mut rx_buf = [0; 4];

    // uart.read_exact_timeout(&mut hdr, timeout_ms)?;
    uart.read(&mut rx_buf)?;

    println!("Rx buf read: {:?}", rx_buf);

    let len = u16::from_le_bytes([rx_buf[2], rx_buf[3]]) as usize;

    Ok(WifiMode::A)
}