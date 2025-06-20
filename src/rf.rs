//! This module contains Wi-Fi and BLE-specific functionality.

use core::default;

use defmt::{Format, println};
use heapless::{String, Vec};

use crate::{
    AP_BUF_MAX, EspError, RpcReqWifiInit, TX_BUF, WifiInitConfig,
    header::HEADER_SIZE,
    proto_data::RpcId,
    rpc::{RPC_MIN_SIZE, Rpc, setup_rpc},
};
use crate::rpc::{write_rpc_var, WireType};

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

#[derive(Clone, Copy, Format)]
pub enum WifiMode {
    A,
}


pub fn get_sta_list<W>(mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiApGetStaList, uid);

    unsafe {
        let frame_len = setup_rpc(&mut TX_BUF, &rpc, &[]);
        write(&TX_BUF[..frame_len])?;
    }
    Ok(())
}

/// Returns size written.
pub fn get_wifi_mode_pl(buf: &mut [u8], uid: u32) -> usize {
    let data = [];

    let data_len = 0;
    let rpc = Rpc::new_req(RpcId::ReqGetWifiMode, uid);

    let frame_len = setup_rpc(buf, &rpc, &data[..data_len]);

    frame_len
}

pub fn get_wifi_mode<W>(mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    unsafe {
        let frame_len = get_wifi_mode_pl(&mut TX_BUF, uid);
        write(&TX_BUF[..frame_len])?;
    }

    Ok(())
}

// todo: Macros may help.

/// Returns size written.
pub fn wifi_init_pl(buf: &mut [u8], uid: u32, cfg: &WifiInitConfig) -> usize {
    let rpc = Rpc::new_req(RpcId::ReqWifiInit, 0);

    // todo: A/R.
    let mut data = [0; 85]; // cfg size is ~50.

    let pl = RpcReqWifiInit {
        cfg: cfg.clone(), // todo: Don't clone
    };

    let data_len = pl.to_bytes(&mut data);

    let frame_len = setup_rpc(buf, &rpc, &data[..data_len]);

    frame_len
}

// todo: Is this only for station/AP mode.
pub fn wifi_init<W>(mut write: W, cfg: &WifiInitConfig) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    unsafe {
        let frame_len = wifi_init_pl(&mut TX_BUF, 0, &cfg);
        write(&TX_BUF[..frame_len])?;
    }

    Ok(())
}

pub fn wifi_start<W>(mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiStart, uid);

    unsafe {
        let frame_len = setup_rpc(&mut TX_BUF, &rpc, &[]);
        write(&TX_BUF[..frame_len])?;
    }

    Ok(())
}

pub fn wifi_stop<W>(mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiStop, uid);

    unsafe {
        let frame_len = setup_rpc(&mut TX_BUF, &rpc, &[]);
        write(&TX_BUF[..frame_len])?;
    }

    Ok(())
}

pub fn wifi_get_protocol<W>(mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiGetProtocol, uid);

    let mut data = [0; 4];
    let mut i = 0;

    let interface_num = 0; // todo?
    write_rpc_var(&mut data, 1, WireType::Varint, interface_num, &mut i);
    let data_len = i;

    unsafe {
        let frame_len = setup_rpc(&mut TX_BUF, &rpc, &data[..data_len]);
        write(&TX_BUF[..frame_len])?;
    }

    Ok(())
}

