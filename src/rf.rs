//! This module contains Wi-Fi and BLE-specific functionality.

use heapless::{String, Vec};

use crate::{CRC_LEN, Command, EspError, Module, RPC_HEADER_SIZE, Uart, build_frame, calc_crc};

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

// todo: BLE scan (get_ble) is identical – just swap Module::Ble, BleScanStart, BleScanResult, and the payload layout (address + adv-data).
pub fn get_aps(
    uart: &mut Uart,
    timeout_ms: u32,
) -> Result<Vec<ApInfo, { crate::AP_BUF_MAX }>, EspError> {
    let mut out = Vec::<ApInfo, { crate::AP_BUF_MAX }>::new();

    // 1 → start scan
    let mut tx = [0u8; RPC_HEADER_SIZE + CRC_LEN];
    let frame = build_frame(&mut tx, Module::Wifi, Command::WifiScanStart, &[]);
    uart.write(frame)?;

    // 2 → collect results
    loop {
        // read header
        let mut hdr = [0u8; RPC_HEADER_SIZE];
        // uart.read_exact_timeout(&mut hdr, timeout_ms)?;
        uart.read(&mut hdr)?;

        let len = u16::from_le_bytes([hdr[2], hdr[3]]) as usize;

        // filter out possible Ctrl ACKs
        if hdr[4] == Module::Ctrl as u8 {
            // todo: Sloppy to discard byte swith static len buf.
            let mut temp_buf = [0; 100];
            uart.read(&mut temp_buf[..len + CRC_LEN])?; // helper that reads & drops
            continue;
        }

        // sanity: expect WifiScanResult
        if hdr[4] != Module::Wifi as u8 || hdr[5] != Command::WifiScanResult as u8 {
            return Err(EspError::UnexpectedResponse(hdr[5]));
        }

        // read payload + CRC
        let mut buf = [0u8; 256];
        // uart.read_exact_timeout(&mut buf[..len + CRC_LEN], timeout_ms)?;
        uart.read(&mut buf[..len + CRC_LEN])?;

        // verify CRC
        let mut full = [0u8; RPC_HEADER_SIZE + 256];
        full[..RPC_HEADER_SIZE].copy_from_slice(&hdr);
        full[RPC_HEADER_SIZE..RPC_HEADER_SIZE + len + CRC_LEN]
            .copy_from_slice(&buf[..len + CRC_LEN]);
        let rx_crc = u16::from_le_bytes(buf[len..len + 2].try_into().unwrap());
        if calc_crc(&full[..RPC_HEADER_SIZE + len]) != rx_crc {
            return Err(EspError::CrcMismatch);
        }

        // parse payload
        let entries = buf[0] as usize;
        if entries == 0 {
            break;
        } // end-of-scan
        let mut idx = 1;
        for _ in 0..entries {
            let rssi = buf[idx] as i8;
            idx += 1;
            let mut bssid = [0u8; 6];
            bssid.copy_from_slice(&buf[idx..idx + 6]);
            idx += 6;
            let slen = buf[idx] as usize;
            idx += 1;
            let ssid_bytes = &buf[idx..idx + slen];
            idx += slen;

            let mut ssid = String::<32>::new();
            ssid.push_str(core::str::from_utf8(ssid_bytes).unwrap())
                .unwrap();
            out.push(ApInfo { ssid, bssid, rssi }).ok();
        }
    }

    Ok(out)
}
