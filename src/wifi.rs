//! This module contains Wi-Fi and BLE-specific functionality.

use defmt::{println, Format};
use heapless::Vec;

use crate::{
    proto_data::{RpcId, RpcReqWifiInit, RpcReqWifiScanStart},
    rpc::{setup_rpc, write_rpc, Rpc, WireType},
    util::write_empty_msg,
    EspError,
};
use crate::proto_data::WifiHeApInfo;
use crate::rpc::decode_varint;
use crate::WireType::{Len, Varint};
// todo: Macros may help.

// /// Information about one Wi-Fi access point
// #[derive(Debug)]
// pub struct ApInfo {
//     pub ssid: String<32>,
//     pub bssid: [u8; 6],
//     pub rssi: i8,
// }
//
// /// Information about one BLE advertisement
// #[derive(Debug)]
// pub struct BleDevice {
//     pub addr: [u8; 6],
//     pub data: Vec<u8, 31>,
//     pub rssi: i8,
// }


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

// ---------- WiFi Country ----------
// #[derive(Format)]
#[derive(Default)]
pub struct WifiCountry {
    pub cc: Vec<u8, 30>,
    pub schan: u32,
    pub nchan: u32,
    pub max_tx_power: i32,
    pub policy: i32,
}

// ---------- WiFi Active Scan Time ----------
#[derive(Default, Format)]
pub struct ActiveScanTime {
    /// 0 means use built-ins.
    pub min: u32,
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

// ---------- WiFi Scan Time ----------
// #[derive(Default, Format)]
#[derive(Default)]
pub struct ScanTime {
    pub active: ActiveScanTime,
    /// 0 means use default of 360ms.
    pub passive: u32,
}

impl ScanTime {
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        let mut i = 0;

        // todo size?
        let mut scan_time_buf = [0; 6];
        let active_size = self.active.to_bytes(&mut scan_time_buf);

        write_rpc(buf, 1, Len, active_size as u64, &mut i);
        buf[i..i + active_size].copy_from_slice(&scan_time_buf[..active_size]);
        i += active_size;

        write_rpc(buf, 2, Varint, self.passive as u64, &mut i);

        i
    }
}

// ---------- WiFi Scan Config ----------
// #[derive(Default, Format)]
#[derive(Default)]
pub struct ScanConfig {
    /// Can limit to a specific SSID or MAC. Empty means no filter.
    pub ssid: Vec<u8, 33>,
    pub bssid: Vec<u8, 6>,
    /// 0 means no filter.
    pub channel: u32,
    pub show_hidden: bool,
    /// 0 means active. 1 is passive. 2 is follow.
    pub scan_type: i32,
    pub scan_time: ScanTime,
    pub home_chan_dwell_time: u32,
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
        let mut scan_time_buf = [0; 8];
        let scan_time_size = self.scan_time.to_bytes(&mut scan_time_buf);

        write_rpc(buf, 6, Len, scan_time_size as u64, &mut i);
        buf[i..i + scan_time_size].copy_from_slice(&scan_time_buf[..scan_time_size]);
        i += scan_time_size;

        write_rpc(buf, 7, Varint, self.home_chan_dwell_time as u64, &mut i);

        i
    }
}

// #[derive(Format)]
pub struct RpcRespWifiScanGetApRecords {
    pub number: u32,
    pub ap_records: Vec<WifiApRecord, 50>,
}

// #[derive(Format)]
#[derive(Default)]
/// [docs](WifiApRecord)
pub struct WifiApRecord {
    pub bssid: Vec<u8, 6>,
    pub ssid: Vec<u8, 33>,
    pub primary: u32,
    pub second: i32,
    pub rssi: i32,
    pub authmode: i32,
    pub pairwise_cipher: i32,
    pub group_cipher: i32,
    /// Antenna used to receive beacon from AP
    pub ant: u8,
    pub bitmask: u32,
    pub country: WifiCountry,
    pub he_ap: WifiHeApInfo,
    ///For AP 20 MHz this value is set to 1. For AP 40 MHz this value is set to 2.
    ///  For AP 80 MHz this value is set to 3. For AP 160MHz this value is set to 4.
    ///     For AP 80+80MHz this value is set to 5
    pub bandwidth: u32,
    ///This fields are used only AP bandwidth is 80 and 160 MHz, to transmit the center channel
    ///   frequency of the BSS. For AP bandwidth is 80 + 80 MHz, it is the center channel frequency
    ///    of the lower frequency segment.
    pub vht_ch_freq1: u32,
    ///This fields are used only AP bandwidth is 80 + 80 MHz, and is used to transmit the center
    ///   channel frequency of the second segment.
    pub vht_ch_freq2: u32,
}

impl WifiApRecord {
    pub fn from_bytes(buf: &[u8]) -> Result<(Self, usize), EspError> {
        let mut i = 0;

        // Note: this assumes fields are passed in order.
        let (tag_bssid, len_bssid_tag) = decode_varint(&buf[i..])?;
        i += len_bssid_tag;

        let (len_bssid, bssid_len_len) = decode_varint(&buf[i..])?;
        i += bssid_len_len;

        if i + len_bssid as usize >= buf.len() {
            return Err(EspError::InvalidData);
        }

        let mut bssid = Vec::<_, 6>::from_slice(&buf[i..i + len_bssid as usize]).map_err(|e| EspError::InvalidData)?;
        i += len_bssid as usize;

        let (tag_ssid, len_ssid_tag) = decode_varint(&buf[i..])?;
        i += len_ssid_tag;

        let (len_ssid, ssid_len_len) = decode_varint(&buf[i..])?;
        i += ssid_len_len;

        if i + len_ssid as usize >= buf.len() {
            return Err(EspError::InvalidData);
        }
        let mut ssid = Vec::<_, 33>::from_slice(&buf[i..i + len_ssid as usize]).map_err(|e| EspError::InvalidData)?;
        i += len_ssid as usize;

        let result = Self {
            bssid,
            ssid,
            ..Default::default() // todo: Fill in fields A/R
        };


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
pub fn start<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiStart)
}

/// Stop WiFi If mode is WIFI_MODE_STA, it stops station and frees station control block If mode is WIFI_MODE_AP,
/// it stops soft-AP and frees soft-AP control block If mode is WIFI_MODE_APSTA, it stops station/soft-AP and frees
/// station/soft-AP control block If mode is WIFI_MODE_NAN, it stops NAN and frees NAN control block.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv413esp_wifi_stopv)
pub fn stop<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiStop)
}

/// Get number of APs found in last scan.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv424esp_wifi_scan_get_ap_numP8uint16_t)
pub fn scan_get_ap_num<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiScanGetApNum)
}

/// Get one AP record from the scanned AP list.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv428esp_wifi_scan_get_ap_recordsP8uint16_tP16wifi_ap_record_t)
pub fn scan_get_ap_record<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiScanGetApRecord)
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

pub fn ap_get_sta_list<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiApGetStaList)
}

pub fn get_mode<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
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
pub fn scan_stop<W>(buf: &mut [u8], mut write: W, uid: u32) -> Result<(), EspError>
where
    W: FnMut(&[u8]) -> Result<(), EspError>,
{
    write_empty_msg(buf, write, uid, RpcId::ReqWifiScanStop)
}

/// Set the supported WiFi protocols for the specified interface.
/// [docs](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_wifi.html#_CPPv422esp_wifi_set_protocols16wifi_interface_tP16wifi_protocols_t)
/// interface (ifx) should be 0 for Station, and 1 for Ap.
/// Bitmap: e.g 1 | 2 | 4; = 11B | 11G | 11N. Note that this is the default.
pub fn set_protocol<W>(
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
