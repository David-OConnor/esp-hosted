//! This module contains Wi-Fi and BLE-specific functionality.

use heapless::{String, Vec};

use crate::{
    EspError, Module, build_frame,
    protocol::{HEADER_SIZE, RPC_HEADER_MAX_SIZE},
    rpc::{RpcHeader, setup_rpc},
    rpc_enums::RpcId,
    transport::compute_checksum,
};
#[cfg(feature = "hal")]
use crate::{Uart, UartError};

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
) -> Result<Vec<ApInfo, { crate::AP_BUF_MAX }>, EspError> {
    const FRAME_LEN: usize = HEADER_SIZE + RPC_HEADER_MAX_SIZE + 7; // todo?
    let mut frame_buf = [0; FRAME_LEN];

    let iface_num = 0; // todo: or 1?
    let data = [iface_num];

    let rpc_hdr = RpcHeader {
        id: RpcId::ReqWifiApGetStaList,
        len: 1, // Payload len of 1: Interface number.
    };

    setup_rpc(&mut frame_buf, &rpc_hdr, &data);
    uart.write(&frame_buf)?;

    // // 2 → collect results
    // loop {
    //     // read header
    //     let mut hdr = [0u8; HEADER_SIZE];
    //     // uart.read_exact_timeout(&mut hdr, timeout_ms)?;
    //     uart.read(&mut hdr)?;
    //
    //     let len = u16::from_le_bytes([hdr[2], hdr[3]]) as usize;
    //
    //     // filter out possible Ctrl ACKs
    //     if hdr[4] == Module::Ctrl as u8 {
    //         // todo: Sloppy to discard byte swith static len buf.
    //         let mut temp_buf = [0; 100];
    //         uart.read(&mut temp_buf[..len])?; // helper that reads & drops
    //         continue;
    //     }
    //
    //     // // sanity: expect WifiScanResult
    //     // if hdr[4] != Module::Wifi as u8 || hdr[5] != Command::WifiScanResult as u8 {
    //     //     return Err(EspError::UnexpectedResponse(hdr[5]));
    //     // }
    //
    //     // read payload + CRC
    //     let mut buf = [0u8; 256];
    //     // uart.read_exact_timeout(&mut buf[..len + CRC_LEN], timeout_ms)?;
    //     uart.read(&mut buf[..len])?;
    //
    //     // verify CRC
    //     let mut full = [0u8; HEADER_SIZE + 256];
    //     full[..HEADER_SIZE].copy_from_slice(&hdr);
    //     full[HEADER_SIZE..HEADER_SIZE + len].copy_from_slice(&buf[..len]);
    //     let rx_crc = u16::from_le_bytes(buf[len..len + 2].try_into().unwrap());
    //     if compute_checksum(&full[..HEADER_SIZE + len]) != rx_crc {
    //         return Err(EspError::CrcMismatch);
    //     }
    //
    //     // parse payload
    //     let entries = buf[0] as usize;
    //     if entries == 0 {
    //         break;
    //     } // end-of-scan
    //     let mut idx = 1;
    //     for _ in 0..entries {
    //         let rssi = buf[idx] as i8;
    //         idx += 1;
    //         let mut bssid = [0u8; 6];
    //         bssid.copy_from_slice(&buf[idx..idx + 6]);
    //         idx += 6;
    //         let slen = buf[idx] as usize;
    //         idx += 1;
    //         let ssid_bytes = &buf[idx..idx + slen];
    //         idx += slen;
    //
    //         let mut ssid = String::<32>::new();
    //         ssid.push_str(core::str::from_utf8(ssid_bytes).unwrap())
    //             .unwrap();
    //         out.push(ApInfo { ssid, bssid, rssi }).ok();
    //     }
    // }

    Err(EspError::Timeout)
    // Ok(out)
}
