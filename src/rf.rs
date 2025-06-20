//! This module contains Wi-Fi and BLE-specific functionality.

use micropb::{PbRead, PbDecoder, PbWrite, PbEncoder, MessageEncode, MessageDecode};

use core::default;

use defmt::{Format, println};
use heapless::{String, Vec};
use crate::{AP_BUF_MAX, EspError, RpcReqWifiInit, TX_BUF, WifiInitConfig, header::HEADER_SIZE, proto_data::RpcId, rpc::{RPC_MIN_SIZE, Rpc, setup_rpc}, RpcReqWifiScanStart, Rpc_Req_WifiScanStart, RpcType, Rpc_};
use crate::rpc::{setup_rpc_proto, write_rpc, WireType};

use crate::esp_hosted_proto::Rpc as RpcP;
use crate::esp_hosted_proto::RpcId as RpcIdP;
use crate::header::build_frame;

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
pub fn wifi_init<W>(mut write: W, cfg: &WifiInitConfig, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    unsafe {
        let frame_len = wifi_init_pl(&mut TX_BUF, uid, &cfg);
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

// pub fn wifi_scan_start<W>(mut write: W, uid: u32, cfg: &RpcReqWifiScanStart) -> Result<(), EspError>
pub fn _wifi_scan_start_proto<W>(mut write: W, uid: u32, scan_start: Rpc_Req_WifiScanStart) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiScanStart, uid);
    //
    // let mut data = [0; 20];
    // let i = cfg.to_bytes(&mut data);


    // todo: Move to a helper fn that accepts Rpc(P)
    //
    // let mut message = RpcP::default();
    //
    // message.set_msg_type(RpcType::Req);
    // message.set_msg_id(RpcIdP::ReqWifiScanStart);
    // message.set_uid(uid);
    //
    // message.payload = Some(Rpc_::Payload::ReqWifiScanStart(scan_start));
    //
    // unsafe {
    //     let frame_len = setup_rpc_proto(&mut TX_BUF, &message);
    //     write(&TX_BUF[..frame_len])?;
    // }

    Ok(())
}

pub fn wifi_scan_start<W>(mut write: W, uid: u32, scan_start: &RpcReqWifiScanStart) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiScanStart, uid);

    let mut data = [0; 100];

    let data_size = scan_start.to_bytes(&mut data);

    unsafe {
        let frame_len = setup_rpc(&mut TX_BUF, &rpc, &data[..data_size]);
        write(&TX_BUF[..frame_len])?;
    }

    Ok(())
}

pub fn wifi_set_protocol<W>(mut write: W, uid: u32, ifx: i32, bitmap: i32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiSetProtocol, uid);

    let mut data = [0; 6];

    let mut i = 0;
    write_rpc(&mut data, 1, WireType::Varint, ifx as u64, &mut i);
    write_rpc(&mut data, 2, WireType::Varint, bitmap as u64, &mut i);

    unsafe {
        let frame_len = setup_rpc(&mut TX_BUF, &rpc, &data[0..i]);
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
    write_rpc(&mut data, 1, WireType::Varint, interface_num, &mut i);
    let data_len = i;

    unsafe {
        let frame_len = setup_rpc(&mut TX_BUF, &rpc, &data[..data_len]);
        write(&TX_BUF[..frame_len])?;
    }

    Ok(())
}

