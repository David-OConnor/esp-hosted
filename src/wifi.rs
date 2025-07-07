//! This module contains Wi-Fi and BLE-specific functionality.

use defmt::{Format, println};
use heapless::Vec;
use num_enum::TryFromPrimitive;

use crate::{
    EspError,
    WireType::{Len, Varint},
    proto_data::{RpcId, RpcReqWifiInit, RpcReqWifiScanStart},
    rpc::{Rpc, WireType, decode_tag, decode_varint, setup_rpc, write_rpc},
    util::write_empty_msg,
};
// todo: Macros may help.

const MAX_AP_RECORDS: usize = 30; // todo: A/R.

// /// Information about one BLE advertisement
// #[derive(Debug)]
// pub struct BleDevice {
//     pub addr: [u8; 6],
//     pub data: Vec<u8, 31>,
//     pub rssi: i8,
// }

/// From a data buffer (e.g. as part of the Rpc struct), parse into Access Point records.
/// The data buffer passed starts post the RPC "header"; the same data we include in the `Rpc` struct.
pub fn parse_ap_records(data: &[u8]) -> Result<Vec<WifiApRecord, MAX_AP_RECORDS>, EspError> {
    let mut result = Vec::new();

    let mut i = 1; // todo: Not robust!
    if data.len() == 0 {
        println!("Empty data on parsing AP records.");
        return Err(EspError::InvalidData);
    }

    let (num_records, nr_len) = decode_varint(&data[i..])?;
    i += nr_len;

    for _ in 0..num_records {
        i += 1; // todo: Skipping over the tag for the records struct.

        if i >= data.len() {
            return Err(EspError::InvalidData);
        }

        let (record_len, record_len_len) = decode_varint(&data[i..])?;
        i += record_len_len;

        // todo: This won't work; you need to get the varint size of each field etc!
        let (record, _record_size) = WifiApRecord::from_bytes(&data[i..i + record_len as usize])?;
        i += record_len as usize;

        result.push(record).map_err(|_| EspError::Capacity)?;
    }

    Ok(result)
}

#[derive(Clone, Format)]
pub struct InitConfig {
    pub static_rx_buf_num: i32,
    pub dynamic_rx_buf_num: i32,
    pub tx_buf_type: i32,
    pub static_tx_buf_num: i32,
    pub dynamic_tx_buf_num: i32,
    pub cache_tx_buf_num: i32,
    //
    pub csi_enable: i32,
    pub ampdu_rx_enable: i32,
    pub ampdu_tx_enable: i32,
    pub amsdu_tx_enable: i32,
    //
    pub nvs_enable: i32,
    pub nano_enable: i32,
    pub rx_ba_win: i32,
    //
    pub wifi_task_core_id: i32,
    pub beacon_max_len: i32,
    pub mgmt_sbuf_num: i32,
    pub feature_caps: u64,
    pub sta_disconnected_pm: bool,
    pub espnow_max_encrypt_num: i32,
    pub magic: i32,
}

impl Default for InitConfig {
    /// Suitable for use as an AP (Station)
    fn default() -> Self {
        Self {
            static_rx_buf_num: 10,
            dynamic_rx_buf_num: 32,
            tx_buf_type: 3, // dynamics
            static_tx_buf_num: 0,
            dynamic_tx_buf_num: 32,
            cache_tx_buf_num: 32,
            //
            csi_enable: 0,
            ampdu_rx_enable: 1,
            ampdu_tx_enable: 1,
            amsdu_tx_enable: 1,
            //
            nvs_enable: 1, // enable if using WiFi persistence
            nano_enable: 0,
            rx_ba_win: 6, // default block ack window
            //
            wifi_task_core_id: 0,
            beacon_max_len: 752, // AP beacon max len
            mgmt_sbuf_num: 32,
            feature_caps: 0,
            sta_disconnected_pm: false,
            espnow_max_encrypt_num: 7,
            magic: 0x1F2F3F4F, // todo: QC this.
        }
    }
}

impl InitConfig {
    /// Suitable for passive use
    pub fn new_promiscuous() -> Self {
        Self {
            static_rx_buf_num: 10,
            dynamic_rx_buf_num: 64,
            tx_buf_type: 1,
            static_tx_buf_num: 0,
            dynamic_tx_buf_num: 32, // todo? 0 is giving out of range error
            cache_tx_buf_num: 0,
            //
            csi_enable: 0,
            ampdu_rx_enable: 0,
            ampdu_tx_enable: 0,
            amsdu_tx_enable: 0,
            //
            nvs_enable: 0, // enable if using WiFi persistence
            nano_enable: 0,
            rx_ba_win: 6, // default block ack window
            //
            wifi_task_core_id: 0,
            beacon_max_len: 752, // AP beacon max len
            mgmt_sbuf_num: 32,
            feature_caps: 0,
            sta_disconnected_pm: false,
            espnow_max_encrypt_num: 0,
            magic: 0x1F2F3F4F, // todo: QC this.
        }
    }

    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        let c = &self;
        let v = WireType::Varint;

        let mut i = 0;

        write_rpc(buf, 1, v, c.static_rx_buf_num as u64, &mut i);
        write_rpc(buf, 2, v, c.dynamic_rx_buf_num as u64, &mut i);
        write_rpc(buf, 3, v, c.tx_buf_type as u64, &mut i);
        write_rpc(buf, 4, v, c.static_tx_buf_num as u64, &mut i);
        write_rpc(buf, 5, v, c.dynamic_tx_buf_num as u64, &mut i);
        write_rpc(buf, 6, v, c.cache_tx_buf_num as u64, &mut i);
        write_rpc(buf, 7, v, c.csi_enable as u64, &mut i);
        write_rpc(buf, 8, v, c.ampdu_rx_enable as u64, &mut i);
        write_rpc(buf, 9, v, c.ampdu_tx_enable as u64, &mut i);
        write_rpc(buf, 10, v, c.amsdu_tx_enable as u64, &mut i);
        write_rpc(buf, 11, v, c.nvs_enable as u64, &mut i);
        write_rpc(buf, 12, v, c.nano_enable as u64, &mut i);
        write_rpc(buf, 13, v, c.rx_ba_win as u64, &mut i);
        write_rpc(buf, 14, v, c.wifi_task_core_id as u64, &mut i);
        write_rpc(buf, 15, v, c.beacon_max_len as u64, &mut i);
        write_rpc(buf, 16, v, c.mgmt_sbuf_num as u64, &mut i);
        write_rpc(buf, 17, v, c.feature_caps, &mut i);
        write_rpc(buf, 18, v, c.sta_disconnected_pm as u64, &mut i);
        write_rpc(buf, 19, v, c.espnow_max_encrypt_num as u64, &mut i);
        write_rpc(buf, 20, v, c.magic as u64, &mut i);

        i
    }
}

#[derive(Clone, Default, Format)]
/// Range of active scan times per channel.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv423wifi_active_scan_time_t)
pub struct ActiveScanTime {
    /// Minimum active scan time per channel, units: millisecond. 0 means use built-ins.
    pub min: u32,
    /// Maximum active scan time per channel, units: millisecond, values above 1500 ms may cause
    /// station to disconnect from AP and are not recommended.
    pub max: u32,
}

impl ActiveScanTime {
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        let mut i = 0;

        write_rpc(buf, 1, Varint, self.min as u64, &mut i);
        write_rpc(buf, 2, Varint, self.max as u64, &mut i);

        i
    }
}

/// Aggregate of active & passive scan time per channel.
/// [docs][https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv416wifi_scan_time_t)
#[derive(Clone, Default)]
pub struct ScanTime {
    pub active: ActiveScanTime,
    /// Passive scan time per channel, units: millisecond, values above 1500 ms may
    /// cause station to disconnect from AP and are not recommended.
    pub passive: u32,
}

impl ScanTime {
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        let mut i = 0;

        // todo size?
        let mut scan_time_buf = [0; 8]; // Measured at 5 with u16 values.
        let active_size = self.active.to_bytes(&mut scan_time_buf);

        write_rpc(buf, 1, Len, active_size as u64, &mut i);
        buf[i..i + active_size].copy_from_slice(&scan_time_buf[..active_size]);
        i += active_size;

        write_rpc(buf, 2, Varint, self.passive as u64, &mut i);

        i
    }
}

#[derive(Clone, Copy, PartialEq, Default, TryFromPrimitive, Format)]
#[repr(u8)]
pub enum ScanType {
    #[default]
    Active = 0,
    Passive = 1,
}

/// Parameters for an SSID scan.
/// Note: If setting most of these values to 0 or empty Vecs, ESP will use its default settings.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_scan_config_t)
#[derive(Default)]
pub struct ScanConfig {
    /// Can limit to a specific SSID or MAC. Empty means no filter.
    pub ssid: Vec<u8, 33>,
    pub bssid: Vec<u8, 6>,
    /// Channel, scan the specific channel. 0 means no filter.
    pub channel: u8,
    /// Enable it to scan AP whose SSID is hidden
    pub show_hidden: bool,
    /// Scan type, active or passive. 0 means active. 1 is passive. 2 is follow.
    pub scan_type: ScanType,
    pub scan_time: ScanTime,
    pub home_chan_dwell_time: u8,
}

impl ScanConfig {
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        let mut i = 0;

        write_rpc(buf, 1, Len, self.ssid.len() as u64, &mut i);
        buf[i..i + self.ssid.len()].copy_from_slice(&self.ssid);
        i += self.ssid.len();

        write_rpc(buf, 2, Len, self.bssid.len() as u64, &mut i);
        buf[i..i + self.bssid.len()].copy_from_slice(&self.bssid);
        i += self.bssid.len();

        write_rpc(buf, 3, Varint, self.channel as u64, &mut i);
        write_rpc(buf, 4, Varint, self.show_hidden as u64, &mut i);
        write_rpc(buf, 5, Varint, self.scan_type as u64, &mut i);

        // todo size?
        let mut scan_time_buf = [0; 14]; // Measured at 10 with all fields configured as u16.
        let scan_time_size = self.scan_time.to_bytes(&mut scan_time_buf);

        write_rpc(buf, 6, Len, scan_time_size as u64, &mut i);
        buf[i..i + scan_time_size].copy_from_slice(&scan_time_buf[..scan_time_size]);
        i += scan_time_size;

        write_rpc(buf, 7, Varint, self.home_chan_dwell_time as u64, &mut i);

        i
    }
}

/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_second_chan_t)
#[derive(Clone, Copy, PartialEq, Default, Format, TryFromPrimitive)]
#[repr(u8)]
pub enum WifiSecondChan {
    #[default]
    None = 0,
    Above = 1,
    Below = 2,
}

/// Wi-Fi authmode type Strength of authmodes Personal Networks : OPEN < WEP < WPA_PSK < OWE < WPA2_PSK =
/// WPA_WPA2_PSK < WAPI_PSK < WPA3_PSK = WPA2_WPA3_PSK = DPP Enterprise Networks : WIFI_AUTH_WPA2_ENTERPRISE
/// < WIFI_AUTH_WPA3_ENTERPRISE = WIFI_AUTH_WPA2_WPA3_ENTERPRISE < WIFI_AUTH_WPA3_ENT_192.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv416wifi_auth_mode_t)
#[derive(Clone, Copy, PartialEq, Default, Format, TryFromPrimitive)]
#[repr(u8)]
pub enum WifiAuthMode {
    #[default]
    Open = 0,
    WEP = 1,
    WPA_PSK = 2,
    WPA2_PSK = 3,
    WPA_WPA2_PSK = 4,
    ENTERPRISE = 5,
    WPA2_ENTERPRISE = 6,
    WPA3_PSK = 7,
    // todo: MOre A/R.
}

/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_cipher_type_t)
#[derive(Clone, Copy, PartialEq, Default, Format, TryFromPrimitive)]
#[repr(u8)]
pub enum WifiCipher {
    #[default]
    None = 0,
    WEP40 = 1,
    WEP104 = 2,
    TKIP = 3,
    CCMP = 4,
    TKIP_CCMP = 5,
    AES_CMAC128 = 6,
    SMS4 = 7,
    GCMP = 8,
    GCMP256 = 9,
    AES_GMAC128 = 10,
    AES_GMAC256 = 11,
    UNKNOWN = 12, // todo: MOre A/R.
}

/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv410wifi_ant_t)
#[derive(Clone, Copy, PartialEq, Default, Format, TryFromPrimitive)]
#[repr(u8)]
pub enum WifiAnt {
    #[default]
    Ant0 = 0,
    Ant1 = 1,
    /// Invalid
    Max = 2,
}

/// Structure describing Wi-Fi country-based regional restrictions.
/// [docs][https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv414wifi_country_t]
#[derive(Default, Format)]
pub struct WifiCountry {
    /// Country code string.
    pub cc: [u8; 3],
    /// Start channel of the allowed 2.4GHz Wi-Fi channels
    pub schan: u8,
    /// Total channel number of the allowed 2.4GHz Wi-Fi channels
    pub nchan: u8,
    pub max_tx_power: i8,
    /// Enum. Auto for 0, Manual for 1
    pub policy: u8,
}

/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv416wifi_bandwidth_t)
#[derive(Clone, Copy, PartialEq, Default, Format, TryFromPrimitive)]
#[repr(u8)]
pub enum WifiBandwidth {
    #[default]
    HT20 = 0,
    /// 20 Mhz
    BW20 = 1,
    BW_HT40 = 2,
    /// 40 Mhz
    BW40 = 3,
    BW80 = 4,
    BW160 = 5,
    /// 80 + 80 Mhz
    BW80_BW80 = 6,
}

/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv417wifi_he_ap_info_t)
#[derive(Default, Format)]
// todo: The protobuf here doesn't match teh normal docs version
pub struct WifiHeApInfo {
    pub bitmask: u32,
    pub bssid_index: u32,
}

///Description of a Wi-Fi AP.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv416wifi_ap_record_t)
// #[derive(Format)]
#[derive(Default)]
pub struct WifiApRecord {
    pub bssid: [u8; 6],
    pub ssid: Vec<u8, 33>,
    /// Channel of AP
    pub primary: u8,
    pub second: WifiSecondChan,
    pub rssi: i8,
    pub authmode: WifiAuthMode,
    pub pairwise_cipher: WifiCipher,
    pub group_cipher: WifiCipher,
    /// Antenna used to receive beacon from AP
    pub ant: WifiAnt,
    /// Bit 0, 11b. 1: 11g. 2: 11n. 3: low rate. 4-6: 11ax mode.
    pub bitmask: u32,
    pub country: WifiCountry,
    pub he_ap: WifiHeApInfo,
    ///For AP 20 MHz this value is set to 1. For AP 40 MHz this value is set to 2.
    ///  For AP 80 MHz this value is set to 3. For AP 160MHz this value is set to 4.
    ///     For AP 80+80MHz this value is set to 5
    pub bandwidth: WifiBandwidth,
    ///This fields are used only AP bandwidth is 80 and 160 MHz, to transmit the center channel
    ///   frequency of the BSS. For AP bandwidth is 80 + 80 MHz, it is the center channel frequency
    ///    of the lower frequency segment.
    pub vht_ch_freq1: u8,
    ///This fields are used only AP bandwidth is 80 + 80 MHz, and is used to transmit the center
    ///   channel frequency of the second segment.
    pub vht_ch_freq2: u8,
}

impl WifiApRecord {
    pub fn from_bytes(buf: &[u8]) -> Result<(Self, usize), EspError> {
        let mut i = 0;
        let mut result = Self::default();

        loop {
            if i >= buf.len() {
                break;
            }
            let (tag, tag_len) = decode_varint(&buf[i..])?;
            i += tag_len;

            let (field, _wire_type) = decode_tag(tag as u16);

            match field {
                1 => {
                    let (field_len, field_len_len) = decode_varint(&buf[i..])?;
                    i += field_len_len;
                    // println!("BSSID buf: {:?}", &buf[i..i + field_len as usize]);

                    if i + field_len as usize >= buf.len() {
                        return Err(EspError::Capacity);
                    }

                    result
                        .bssid
                        .copy_from_slice(&buf[i..i + field_len as usize]);
                    i += field_len as usize;
                }
                2 => {
                    let (field_len, field_len_len) = decode_varint(&buf[i..])?;
                    i += field_len_len;
                    // println!("SSID buf: {:?}", &buf[i..i + field_len as usize]);

                    if i + field_len as usize >= buf.len() {
                        return Err(EspError::Capacity);
                    }

                    result.ssid = Vec::<_, 33>::from_slice(&buf[i..i + field_len as usize])
                        .map_err(|_| EspError::InvalidData)?;
                    i += field_len as usize;
                }
                3 => {
                    result.primary = buf[i];
                    i += 1;
                }
                4 => {
                    result.second = buf[i].try_into().unwrap_or_default();
                    i += 1;
                }
                5 => {
                    result.rssi = buf[i] as i8;
                    i += 10; // Non-zigzag protobuf negative val encoding... yikes.
                }
                6 => {
                    result.authmode = buf[i].try_into().unwrap_or_default();
                    i += 1;
                }
                7 => {
                    result.pairwise_cipher = buf[i].try_into().unwrap_or_default();
                    i += 1;
                }
                8 => {
                    result.group_cipher = buf[i].try_into().unwrap_or_default();
                    i += 1;
                }
                9 => {
                    result.ant = buf[i].try_into().unwrap_or_default();
                    i += 1;
                }
                10 => {
                    let (val, len) = decode_varint(&buf[i..])?;
                    result.bitmask = val as u32;
                    i += len;
                }
                11 => {
                    let (country_len, country_len_len) = decode_varint(&buf[i..])?;
                    i += country_len_len;
                    // todo: Parse here.
                    i += country_len as usize;
                }
                12 => {
                    let (he_ap_len, he_ap_len_len) = decode_varint(&buf[i..])?;
                    i += he_ap_len_len;
                    // todo: Parse here.
                    i += he_ap_len as usize;
                }
                13 => {
                    result.bandwidth = buf[i].try_into().unwrap_or_default();
                    i += 1;
                }
                14 => {
                    result.vht_ch_freq1 = buf[i];
                    i += 1;
                }
                15 => {
                    result.vht_ch_freq2 = buf[i];
                    i += 1;
                }
                _ => {
                    println!("Unparsed field: {:?}", field);
                }
            }
        }

        Ok((result, i))
    }
}

/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv416wifi_interface_t)
#[derive(Clone, Copy, Format)]
#[repr(u8)]
pub enum InterfaceType {
    Station = 0,
    Ap = 1,
}

/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv411wifi_mode_t)
#[derive(Clone, Copy, Format)]
#[repr(u8)]
pub enum WifiMode {
    Null = 0,
    /// Wi-Fi station mode
    Station = 1,
    /// Wi-Fi soft-AP mode
    SoftAp = 2,
    /// Wi-Fi station + soft-AP mode
    ApStation = 3,
}

/// Start WiFi according to current configuration If mode is WIFI_MODE_STA, it creates station control block and starts station If mode is
/// WIFI_MODE_AP, it creates soft-AP control block and starts soft-AP If mode is WIFI_MODE_APSTA, it creates soft-AP and station control
/// block and starts soft-AP and station If mode is WIFI_MODE_NAN, it creates NAN control block and starts NAN.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv414esp_wifi_startv))
pub fn start<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiStart)
}

/// Stop WiFi If mode is WIFI_MODE_STA, it stops station and frees station control block If mode is WIFI_MODE_AP,
/// it stops soft-AP and frees soft-AP control block If mode is WIFI_MODE_APSTA, it stops station/soft-AP and frees
/// station/soft-AP control block If mode is WIFI_MODE_NAN, it stops NAN and frees NAN control block.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv413esp_wifi_stopv)
pub fn stop<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiStop)
}

/// Get number of APs found in last scan.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv424esp_wifi_scan_get_ap_numP8uint16_t)
pub fn scan_get_ap_num<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiScanGetApNum)
}

/// Get one AP record from the scanned AP list.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv428esp_wifi_scan_get_ap_recordsP8uint16_tP16wifi_ap_record_t)
pub fn scan_get_ap_record<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiScanGetApRecord)
}

/// Clear AP list found in last scan.
/// This API will free all memory occupied by scanned AP list.
/// When the obtained AP list fails, AP records must be cleared,otherwise it may cause memory leakage.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv422esp_wifi_clear_ap_listv)
pub fn clear_ap_list<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiClearApList)
}

/// Retrieve the list of APs found during the last scan. The returned AP list is sorted in descending order based on RSSI.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv428esp_wifi_scan_get_ap_recordsP8uint16_tP16wifi_ap_record_t)
pub fn scan_get_ap_records<W>(
    buf: &mut [u8],
    mut write: W,
    uid: u32,
    max_number: u8,
) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiScanGetApRecords, uid);

    let mut data = [0; 2];

    let mut i = 0;
    write_rpc(&mut data, 1, WireType::Varint, max_number as u64, &mut i);

    let frame_len = setup_rpc(buf, &rpc, &data[..i]);
    write(&buf[..frame_len])?;

    Ok(())
}

pub fn ap_get_sta_list<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiApGetStaList)
}

pub fn get_mode<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
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
pub fn set_mode<W>(buf: &mut [u8], mut write: W, uid: u32, mode: WifiMode) -> Result<(), EspError>
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

/// Initialize WiFi Allocate resource for WiFi driver, such as WiFi control structure, RX/TX buffer,
/// WiFi NVS structure etc. This WiFi also starts WiFi task.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv413esp_wifi_initPK18wifi_init_config_t)
pub fn init<W>(buf: &mut [u8], mut write: W, uid: u32, cfg: &InitConfig) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiInit, uid);

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

/// Deinit WiFi Free all resource allocated in esp_wifi_init and stop WiFi task.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv415esp_wifi_deinitv))
pub fn deinit<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiDeinit)
}

/// Promiscuous frame type.
///
/// Passed to promiscuous mode RX callback to indicate the type of parameter in the buffer.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv427wifi_promiscuous_pkt_type_t)
#[derive(Clone, Copy, PartialEq, Format)]
#[repr(u8)]
pub enum PromiscuousPktType {
    /// Management frame
    Mgmt = 0,
    /// Control frame
    Ctrl = 1,
    /// Data frame
    Data = 2,
    /// Other type, such as MIMO etc.
    Misc = 3,
}

#[derive(Format, Default)]
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#c.WIFI_PROMIS_FILTER_MASK_ALL)
pub struct PromiscuousFilter {
    pub mgmt: bool,
    pub ctrl: bool,
    pub data: bool,
    pub misc: bool,
    pub data_mpdu: bool,
    pub data_ampdu: bool,
    pub fcsfail: bool,
}

impl PromiscuousFilter {
    pub fn val(&self) -> u32 {
        let mut result = 0;
        if self.mgmt {
            result |= 1 << 0;
        }
        if self.ctrl {
            result |= 1 << 1;
        }
        if self.data {
            result |= 1 << 2;
        }
        if self.misc {
            result |= 1 << 3;
        }
        if self.data_mpdu {
            result |= 1 << 4;
        }
        if self.data_ampdu {
            result |= 1 << 5;
        }
        if self.fcsfail {
            result |= 1 << 6;
        }
        result
    }
}

#[derive(Format, Default)]
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#c.WIFI_PROMIS_CTRL_FILTER_MASK_ALL)
pub struct PromiscuousCtrlFilter {
    pub wrapper: bool,
    pub bar: bool,
    pub ba: bool,
    pub pspoll: bool,
    pub rts: bool,
    pub cts: bool,
    pub ack: bool,
    pub cfend: bool,
    pub cfendack: bool,
}

impl PromiscuousCtrlFilter {
    pub fn val(&self) -> u32 {
        let mut result = 0;
        if self.wrapper {
            result |= 1 << 0;
        }
        if self.bar {
            result |= 1 << 1;
        }
        if self.ba {
            result |= 1 << 2;
        }
        if self.pspoll {
            result |= 1 << 3;
        }
        if self.rts {
            result |= 1 << 4;
        }
        if self.cts {
            result |= 1 << 5;
        }
        if self.ack {
            result |= 1 << 6;
        }
        if self.cfend {
            result |= 1 << 7;
        }
        if self.cfendack {
            result |= 1 << 8;
        }
        result
    }
}

/// Enable the promiscuous mode, and set its filter.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv424esp_wifi_set_promiscuousb)
pub fn set_promiscuous<W>(
    buf: &mut [u8],
    mut write: W,
    uid: u32,
    enabled: bool,
    filter: &PromiscuousFilter,
    ctrl_filter: Option<&PromiscuousCtrlFilter>,
) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    // todo: Where to handle setting the CB? Doesn't map neatly to RPC.

    // Enable or disable
    let rpc = Rpc::new_req(RpcId::ReqWifiSetPromiscuous, uid);

    let mut data = [0; 6];

    let mut i = 0;
    write_rpc(&mut data, 1, WireType::Varint, enabled as u64, &mut i);

    let frame_len = setup_rpc(buf, &rpc, &data[..i]);
    write(&buf[..frame_len])?;

    // Set its filter. This, and the ctrl filter, and single-field structs.
    let rpc = Rpc::new_req(RpcId::ReqWifiSetPromiscuousFilter, uid);

    let mut i = 0;
    // Same buf as enable/disable.

    // todo: We assume 1 byte now for each filter. This is temp.
    write_rpc(&mut data, 1, WireType::Len, 1, &mut i);
    write_rpc(&mut data, 1, WireType::Varint, 1, &mut i); // Len of the struct.

    write_rpc(&mut data, 1, WireType::Varint, filter.val() as u64, &mut i);

    let frame_len = setup_rpc(buf, &rpc, &data[..i]);
    write(&buf[..frame_len])?;

    // Set its ctrl-mode filter A/R
    if let Some(f) = ctrl_filter {
        let rpc = Rpc::new_req(RpcId::ReqWifiSetPromiscuousCtrlFilter, uid);

        let mut i = 0;

        write_rpc(&mut data, 1, WireType::Len, 1, &mut i);
        write_rpc(&mut data, 1, WireType::Varint, 1, &mut i); // Len of the struct.

        write_rpc(&mut data, 1, WireType::Varint, f.val() as u64, &mut i);

        let frame_len = setup_rpc(buf, &rpc, &data[..i]);
        write(&buf[..frame_len])?;
    }

    Ok(())
}

/// Get the promiscuous mode.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv424esp_wifi_get_promiscuousPb)
pub fn get_promiscuous<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiGetPromiscuous)
}

/// Get the promiscuous filter.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv431esp_wifi_get_promiscuous_filterP25wifi_promiscuous_filter_t)
pub fn get_promiscuous_filter<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiGetPromiscuousFilter)
}

/// Get the subtype filter of the control packet in promiscuous mode.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv436esp_wifi_get_promiscuous_ctrl_filterP25wifi_promiscuous_filter_t)
pub fn get_promiscuous_ctrl_filter<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiGetPromiscuousCtrlFilter)
}

/// Scan all available APs.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv419esp_wifi_scan_startPK18wifi_scan_config_tb)
pub fn scan_start<W>(
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

/// Stop the scan in process.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv418esp_wifi_scan_stopv)
pub fn scan_stop<W>(buf: &mut [u8], write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiScanStop)
}

#[derive(Format)]
pub struct Protocols {
    /// 802.11b
    pub p_11b: bool,
    /// 802.11g
    pub p_11g: bool,
    /// 802.11n
    pub p_11n: bool,
    /// Long range
    pub p_lr: bool,
    /// 802.11ax
    pub p_11ax: bool,
    /// 802.11ax
    pub wps: bool,
    /// 802.11a
    pub p_11a: bool,
    pub p_11ac: bool,
}

impl Default for Protocols {
    /// This matches the ESP default.
    fn default() -> Self {
        Self {
            p_11b: true,
            p_11g: true,
            p_11n: true,
            p_lr: false,
            p_11ax: false,
            wps: false,
            p_11a: false,
            p_11ac: false,
            // todo: 802.1ac and ax A/R. 0x20 and 0x40 rep
        }
    }
}

impl Protocols {
    pub fn to_byte(&self) -> u8 {
        (self.p_11b as u8)
            | ((self.p_11g as u8) << 1)
            | ((self.p_11n as u8) << 2)
            | ((self.p_lr as u8) << 3)
            | ((self.p_11ac as u8) << 5)
            | ((self.p_11ax as u8) << 6)
    }

    pub fn from_byte(b: u8) -> Self {
        Self {
            p_11b: b & 1 != 0,
            p_11g: (b >> 1) & 1 != 0,
            p_11n: (b >> 2) & 1 != 0,
            p_lr: (b >> 3) & 1 != 0,
            p_11ax: (b >> 4) & 1 != 0,
            wps: (b >> 5) & 1 != 0,
            p_11a: false,
            p_11ac: false,
            // todo?
            // p_11a: (b >> 8) & 1 != 0,
            // p_11ac: (b >> 9) & 1 != 0,
        }
    }
}

/// Set the supported WiFi protocols for the specified interface. The default protocol is
/// (WIFI_PROTOCOL_11B|WIFI_PROTOCOL_11G|WIFI_PROTOCOL_11N).
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv422esp_wifi_set_protocols16wifi_interface_tP16wifi_protocols_t)
/// interface (ifx) should be 0 for Station, and 1 for Ap.
/// Bitmap: e.g 1 | 2 | 4; = 11B | 11G | 11N. Note that this is the default.
pub fn set_protocol<W>(
    buf: &mut [u8],
    mut write: W,
    uid: u32,
    ifx: InterfaceType,
    protocols: &Protocols,
) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    let rpc = Rpc::new_req(RpcId::ReqWifiSetProtocol, uid);

    let mut data = [0; 4];
    let mut i = 0;

    write_rpc(&mut data, 1, WireType::Varint, ifx as u64, &mut i);
    write_rpc(
        &mut data,
        2,
        WireType::Varint,
        protocols.to_byte() as u64,
        &mut i,
    );

    let frame_len = setup_rpc(buf, &rpc, &data[0..i]);
    write(&buf[..frame_len])?;

    Ok(())
}

/// Get the current protocol bitmap of the specified interface.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv421esp_wifi_get_protocol16wifi_interface_tP7uint8_t)
pub fn get_protocol<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
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

/// Set current WiFi power save type.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv415esp_wifi_set_ps14wifi_ps_type_t)
pub fn set_ps<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
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
