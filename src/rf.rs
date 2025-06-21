//! This module contains Wi-Fi and BLE-specific functionality.

use defmt::Format;
use heapless::{String, Vec};

use crate::{
    EspError, RpcReqWifiInit, RpcReqWifiScanStart, WifiInitConfig,
    proto_data::RpcId,
    rpc::{InterfaceType, Rpc, WireType, setup_rpc, write_rpc},
    util::write_empty_msg,
};

// todo: Macros may help.

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

pub fn wifi_start<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiStart)
}

pub fn wifi_stop<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiStop)
}

pub fn wifi_scan_get_ap_num<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiScanGetApNum)
}

pub fn wifi_ap_get_sta_list<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiApGetStaList)
}

pub fn get_wifi_mode<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqGetWifiMode)
}

/// Options:
/// 0: Radio off
/// 1: Station/client: Can scan and connect
/// 2: Soft AP; cannot scan.
/// 3: Soft-AP and Sta (slower scan)
/// 4: Wi-Fi aware. (Not relevant to normal scanning)
pub fn set_wifi_mode<W>(buf: &mut [u8], mut write: W, uid: u32, mode: i32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqSetWifiMode, uid);

    let mut data = [0; 2];

    let mut i = 0;
    write_rpc(&mut data, 1, WireType::Varint, mode as u64, &mut i);

    let frame_len = setup_rpc(buf, &rpc, &data[..i]);
    write(&buf[..frame_len])?;

    Ok(())
}

pub fn wifi_init<W>(
    buf: &mut [u8],
    mut write: W,
    uid: u32,
    cfg: &WifiInitConfig,
) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiInit, 0);

    // todo: A/R.
    let mut data = [0; 85]; // cfg size is ~50.

    let pl = RpcReqWifiInit {
        cfg: cfg.clone(), // todo: Don't clone
    };

    let data_len = pl.to_bytes(&mut data);

    let frame_len = setup_rpc(buf, &rpc, &data[..data_len]);
    write(&buf[..frame_len])?;

    Ok(())
}

// // pub fn wifi_scan_start<W>(buf: &mut [u8], mut write: W, uid: u32, cfg: &RpcReqWifiScanStart) -> Result<(), EspError>
// pub fn _wifi_scan_start_proto<W>(buf: &mut [u8], mut write: W, uid: u32, scan_start: Rpc_Req_WifiScanStart) -> Result<(), EspError>
// where
//     W: FnMut(&[u8]) -> Result<(), EspError>,
// {
//     let rpc = Rpc::new_req(RpcId::ReqWifiScanStart, uid);
//     //
//     // let mut data = [0; 20];
//     // let i = cfg.to_bytes(&mut data);
//
//
//     // todo: Move to a helper fn that accepts Rpc(P)
//     //
//     // let mut message = RpcP::default();
//     //
//     // message.set_msg_type(RpcType::Req);
//     // message.set_msg_id(RpcIdP::ReqWifiScanStart);
//     // message.set_uid(uid);
//     //
//     // message.payload = Some(Rpc_::Payload::ReqWifiScanStart(scan_start));
//     //
//     //     let frame_len = setup_rpc_proto(buf, &message);
//     //     write(&buf[..frame_len])?;
//
//     Ok(())
// }

pub fn wifi_scan_start<W>(
    buf: &mut [u8],
    mut write: W,
    uid: u32,
    scan_start: &RpcReqWifiScanStart,
) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiScanStart, uid);

    let mut data = [0; 100];
    let data_size = scan_start.to_bytes(&mut data);

    let frame_len = setup_rpc(buf, &rpc, &data[..data_size]);
    write(&buf[..frame_len])?;

    Ok(())
}

/// interface (ifx) should be 0 for Station, and 1 for Ap.
/// Bitmap: e.g 1 | 2 | 4; = 11B | 11G | 11N. Note that this is the default.
pub fn wifi_set_protocol<W>(
    buf: &mut [u8],
    mut write: W,
    uid: u32,
    ifx: InterfaceType,
    bitmap: i32,
) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiSetProtocol, uid);

    let mut data = [0; 4];
    let mut i = 0;

    write_rpc(&mut data, 1, WireType::Varint, ifx as u64, &mut i);
    write_rpc(&mut data, 2, WireType::Varint, bitmap as u64, &mut i);

    let frame_len = setup_rpc(buf, &rpc, &data[0..i]);
    write(&buf[..frame_len])?;

    Ok(())
}

pub fn wifi_get_protocol<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiGetProtocol, uid);

    let mut data = [0; 4];
    let mut i = 0;

    let interface_num = 0; // todo?
    write_rpc(&mut data, 1, WireType::Varint, interface_num, &mut i);

    let frame_len = setup_rpc(buf, &rpc, &data[..i]);
    write(&buf[..frame_len])?;

    Ok(())
}
