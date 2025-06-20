//! Contains data types specific to the ESP-Hosted-MCU proto buffer. This contains
//! definitions for the data types [de]serialized.

use defmt::{println, Format};
use heapless::Vec;
use num_enum::TryFromPrimitive;

use crate::rpc::{WireType, write_rpc};
use crate::rpc::WireType::{Len, Varint};

const MAX_DATA_SIZE: usize = 1000; // todo temp

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u16)]
/// See `esp_hosted_rpc.proto`, enum by this name. This is encoded as a varint.
pub enum RpcId {
    MsgIdInvalid = 0,
    ReqBase = 256,
    ReqGetMacAddress = 257,
    ReqSetMacAddress = 258,
    ReqGetWifiMode = 259,
    ReqSetWifiMode = 260,

    ReqWifiSetPs = 270,
    ReqWifiGetPs = 271,

    ReqOtaBegin = 272,
    ReqOtaWrite = 273,
    ReqOtaEnd = 274,

    ReqWifiSetMaxTxPower = 275,
    ReqWifiGetMaxTxPower = 276,

    ReqConfigHeartbeat = 277,

    ReqWifiInit = 278,
    ReqWifiDeinit = 279,
    ReqWifiStart = 280,
    ReqWifiStop = 281,
    ReqWifiConnect = 282,
    ReqWifiDisconnect = 283,
    ReqWifiSetConfig = 284,
    ReqWifiGetConfig = 285,

    ReqWifiScanStart = 286,
    ReqWifiScanStop = 287,
    ReqWifiScanGetApNum = 288,
    ReqWifiScanGetApRecords = 289,
    ReqWifiClearApList = 290,

    ReqWifiRestore = 291,
    ReqWifiClearFastConnect = 292,
    ReqWifiDeauthSta = 293,
    ReqWifiStaGetApInfo = 294,

    ReqWifiSetProtocol = 297,
    ReqWifiGetProtocol = 298,
    ReqWifiSetBandwidth = 299,
    ReqWifiGetBandwidth = 300,
    ReqWifiSetChannel = 301,
    ReqWifiGetChannel = 302,
    ReqWifiSetCountry = 303,
    ReqWifiGetCountry = 304,

    ReqWifiSetPromiscuous = 305,
    ReqWifiGetPromiscuous = 306,
    ReqWifiSetPromiscuousFilter = 307,
    ReqWifiGetPromiscuousFilter = 308,
    ReqWifiSetPromiscuousCtrlFilter = 309,
    ReqWifiGetPromiscuousCtrlFilter = 310,

    ReqWifiApGetStaList = 311,
    ReqWifiApGetStaAid = 312,
    ReqWifiSetStorage = 313,
    ReqWifiSetVendorIe = 314,
    ReqWifiSetEventMask = 315,
    ReqWifiGetEventMask = 316,
    ReqWifi80211Tx = 317,

    ReqWifiSetCsiConfig = 318,
    ReqWifiSetCsi = 319,

    ReqWifiSetAntGpio = 320,
    ReqWifiGetAntGpio = 321,
    ReqWifiSetAnt = 322,
    ReqWifiGetAnt = 323,

    ReqWifiGetTsfTime = 324,
    ReqWifiSetInactiveTime = 325,
    ReqWifiGetInactiveTime = 326,
    ReqWifiStatisDump = 327,
    ReqWifiSetRssiThreshold = 328,

    ReqWifiFtmInitiateSession = 329,
    ReqWifiFtmEndSession = 330,
    ReqWifiFtmRespSetOffset = 331,

    ReqWifiConfig11bRate = 332,
    ReqWifiConnectionlessModuleSetWakeInterval = 333,
    ReqWifiSetCountryCode = 334,
    ReqWifiGetCountryCode = 335,
    ReqWifiConfig80211TxRate = 336,
    ReqWifiDisablePmfConfig = 337,
    ReqWifiStaGetAid = 338,
    ReqWifiStaGetNegotiatedPhymode = 339,
    ReqWifiSetDynamicCs = 340,
    ReqWifiStaGetRssi = 341,

    ReqWifiSetProtocols = 342,
    ReqWifiGetProtocols = 343,
    ReqWifiSetBandwidths = 344,
    ReqWifiGetBandwidths = 345,
    ReqWifiSetBand = 346,
    ReqWifiGetBand = 347,
    ReqWifiSetBandMode = 348,
    ReqWifiGetBandMode = 349,

    ReqGetCoprocessorFwVersion = 350,
    ReqWifiScanGetApRecord = 351,
    ReqMax = 352,

    RespBase = 512,
    RespGetMacAddress = 513,
    RespSetMacAddress = 514,
    RespGetWifiMode = 515,
    RespSetWifiMode = 516,

    RespWifiSetPs = 526,
    RespWifiGetPs = 527,

    RespOtaBegin = 528,
    RespOtaWrite = 529,
    RespOtaEnd = 530,

    RespWifiSetMaxTxPower = 531,
    RespWifiGetMaxTxPower = 532,

    RespConfigHeartbeat = 533,

    RespWifiInit = 534,
    RespWifiDeinit = 535,
    RespWifiStart = 536,
    RespWifiStop = 537,
    RespWifiConnect = 538,
    RespWifiDisconnect = 539,
    RespWifiSetConfig = 540,
    RespWifiGetConfig = 541,

    RespWifiScanStart = 542,
    RespWifiScanStop = 543,
    RespWifiScanGetApNum = 544,
    RespWifiScanGetApRecords = 545,
    RespWifiClearApList = 546,

    RespWifiRestore = 547,
    RespWifiClearFastConnect = 548,
    RespWifiDeauthSta = 549,
    RespWifiStaGetApInfo = 550,

    RespWifiSetProtocol = 553,
    RespWifiGetProtocol = 554,
    RespWifiSetBandwidth = 555,
    RespWifiGetBandwidth = 556,
    RespWifiSetChannel = 557,
    RespWifiGetChannel = 558,
    RespWifiSetCountry = 559,
    RespWifiGetCountry = 560,

    RespWifiSetPromiscuous = 561,
    RespWifiGetPromiscuous = 562,
    RespWifiSetPromiscuousFilter = 563,
    RespWifiGetPromiscuousFilter = 564,
    RespWifiSetPromiscuousCtrlFilter = 565,
    RespWifiGetPromiscuousCtrlFilter = 566,

    RespWifiApGetStaList = 567,
    RespWifiApGetStaAid = 568,
    RespWifiSetStorage = 569,
    RespWifiSetVendorIe = 570,
    RespWifiSetEventMask = 571,
    RespWifiGetEventMask = 572,
    RespWifi80211Tx = 573,

    RespWifiSetCsiConfig = 574,
    RespWifiSetCsi = 575,

    RespWifiSetAntGpio = 576,
    RespWifiGetAntGpio = 577,
    RespWifiSetAnt = 578,
    RespWifiGetAnt = 579,

    RespWifiGetTsfTime = 580,
    RespWifiSetInactiveTime = 581,
    RespWifiGetInactiveTime = 582,
    RespWifiStatisDump = 583,
    RespWifiSetRssiThreshold = 584,

    RespWifiFtmInitiateSession = 585,
    RespWifiFtmEndSession = 586,
    RespWifiFtmRespSetOffset = 587,

    RespWifiConfig11bRate = 588,
    RespWifiConnectionlessModuleSetWakeInterval = 589,
    RespWifiSetCountryCode = 590,
    RespWifiGetCountryCode = 591,
    RespWifiConfig80211TxRate = 592,
    RespWifiDisablePmfConfig = 593,
    RespWifiStaGetAid = 594,
    RespWifiStaGetNegotiatedPhymode = 595,
    RespWifiSetDynamicCs = 596,
    RespWifiStaGetRssi = 597,

    RespWifiSetProtocols = 598,
    RespWifiGetProtocols = 599,
    RespWifiSetBandwidths = 600,
    RespWifiGetBandwidths = 601,
    RespWifiSetBand = 602,
    RespWifiGetBand = 603,
    RespWifiSetBandMode = 604,
    RespWifiGetBandMode = 605,

    RespGetCoprocessorFwVersion = 606,
    RespWifiScanGetApRecord = 607,
    RespMax = 608,

    EventBase = 768,
    EventEspInit = 769,
    EventHeartbeat = 770,
    EventApStaConnected = 771,
    EventApStaDisconnected = 772,
    EventWifiEventNoArgs = 773,
    EventStaScanDone = 774,
    EventStaConnected = 775,
    EventStaDisconnected = 776,
    EventMax = 777,
}

#[derive(Clone, Format)]
pub struct WifiInitConfig {
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

impl Default for WifiInitConfig {
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

impl WifiInitConfig {
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
pub struct WifiCountry {
    pub cc: Vec<u8, 30>,
    pub schan: u32,
    pub nchan: u32,
    pub max_tx_power: i32,
    pub policy: i32,
}

// ---------- WiFi Active Scan Time ----------
#[derive(Format)]
pub struct WifiActiveScanTime {
    pub min: u32,
    pub max: u32,
}

// ---------- WiFi Scan Time ----------
// #[derive(Format)]
pub struct WifiScanTime {
    pub active: WifiActiveScanTime,
    pub passive: u32,
}

// ---------- WiFi Scan Config ----------
// #[derive(Format)]
pub struct WifiScanConfig {
    pub ssid: Vec<u8, 30>,
    pub bssid: Vec<u8, 30>,
    pub channel: u32,
    pub show_hidden: bool,
    pub scan_type: i32,
    pub scan_time: WifiScanTime,
    pub home_chan_dwell_time: u32,
}

// impl WifiScanConfig {
//     pub fn to
// }

// ---------- WiFi HE AP Info ----------
#[derive(Format)]
pub struct WifiHeApInfo {
    pub bitmask: u32,
    pub bssid_index: u32,
}

// ---------- WiFi AP Record ----------
// #[derive(Format)]
pub struct WifiApRecord {
    pub bssid: Vec<u8, 30>,
    pub ssid: Vec<u8, 30>,
    pub primary: u32,
    pub second: i32,
    pub rssi: i32,
    pub authmode: i32,
    pub pairwise_cipher: i32,
    pub group_cipher: i32,
    pub ant: i32,
    pub bitmask: u32,
    pub country: WifiCountry,
    pub he_ap: WifiHeApInfo,
    pub bandwidth: u32,
    pub vht_ch_freq1: u32,
    pub vht_ch_freq2: u32,
}

// ---------- WiFi Scan Threshold ----------
#[derive(Format)]
pub struct WifiScanThreshold {
    pub rssi: i32,
    pub authmode: i32,
}

// ---------- WiFi PMF Config ----------
#[derive(Format)]
pub struct WifiPmfConfig {
    pub capable: bool,
    pub required: bool,
}

// ---------- WiFi AP Config ----------
// #[derive(Format)]
pub struct WifiApConfig {
    pub ssid: Vec<u8, 30>,
    pub password: Vec<u8, 30>,
    pub ssid_len: u32,
    pub channel: u32,
    pub authmode: i32,
    pub ssid_hidden: u32,
    pub max_connection: u32,
    pub beacon_interval: u32,
    pub pairwise_cipher: i32,
    pub ftm_responder: bool,
    pub pmf_cfg: WifiPmfConfig,
    pub sae_pwe_h2e: i32,
}

// ---------- WiFi STA Config ----------
// #[derive(Format)]
pub struct WifiStaConfig {
    pub ssid: Vec<u8, 30>,
    pub password: Vec<u8, 30>,
    pub scan_method: i32,
    pub bssid_set: bool,
    pub bssid: Vec<u8, 30>,
    pub channel: u32,
    pub listen_interval: u32,
    pub sort_method: i32,
    pub threshold: WifiScanThreshold,
    pub pmf_cfg: WifiPmfConfig,
    pub bitmask: u32,
    pub sae_pwe_h2e: i32,
    pub failure_retry_cnt: u32,
    pub he_bitmask: u32,
    pub sae_h2e_identifier: Vec<u8, 30>,
}

// ---------- WiFi Config (oneof) ----------
// #[derive(Format)]
pub enum WifiConfig {
    Ap(WifiApConfig),
    Sta(WifiStaConfig),
}

// ---------- WiFi STA Info ----------
// #[derive(Format)]
pub struct WifiStaInfo {
    pub mac: Vec<u8, 30>,
    pub rssi: i32,
    pub bitmask: u32,
}

// ---------- WiFi STA List ----------
// #[derive(Format)]
pub struct WifiStaList {
    pub sta: Vec<WifiStaInfo, 30>,
    pub num: i32,
}

// ---------- OTA ----------
#[derive(Format)]
pub struct RpcReqOtaBegin;

#[derive(Format)]
pub struct RpcRespOtaBegin {
    pub resp: i32,
}

// #[derive(Format)]
pub struct RpcReqOtaWrite {
    pub ota_data: Vec<u8, MAX_DATA_SIZE>,
}

#[derive(Format)]
pub struct RpcRespOtaWrite {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqOtaEnd;

#[derive(Format)]
pub struct RpcRespOtaEnd {
    pub resp: i32,
}

// ---------- WiFi Power ----------
#[derive(Format)]
pub struct RpcReqWifiSetMaxTxPower {
    pub power: i32,
}

#[derive(Format)]
pub struct RpcRespWifiSetMaxTxPower {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiGetMaxTxPower;

#[derive(Format)]
pub struct RpcRespWifiGetMaxTxPower {
    pub power: i32,
    pub resp: i32,
}

// ---------- Heartbeat ----------
#[derive(Format)]
pub struct RpcReqConfigHeartbeat {
    pub enable: bool,
    /// It appears that this is in intervals of 10s.
    pub duration: i32,
}

impl RpcReqConfigHeartbeat {
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        let mut i = 0;

        write_rpc(buf, 1, WireType::Varint, self.enable as u64, &mut i);
        write_rpc(buf, 2, WireType::Varint, self.duration as u64, &mut i);

        i
    }
}

#[derive(Format)]
pub struct RpcRespConfigHeartbeat {
    pub resp: i32,
}

// ---------- WiFi Init/Deinit ----------
#[derive(Format)]
pub struct RpcReqWifiInit {
    pub cfg: WifiInitConfig,
}

impl RpcReqWifiInit {
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        let c = &self.cfg;
        let v = WireType::Varint;

        // Total size came out to 50 for a test case.
        let mut buf_init_cfg = [0; 80]; // todo: Not a great place for this.

        let cfg_len = self.cfg.to_bytes(&mut buf_init_cfg);
        println!("Wifi init cfg len: {:?}", cfg_len);

        let mut i = 0;
        write_rpc(buf, 1, WireType::Len, cfg_len as u64, &mut i);

        buf[i..i + cfg_len].copy_from_slice(&buf_init_cfg[..cfg_len]);
        i += cfg_len;

        i
    }
}

#[derive(Format)]
pub struct RpcRespWifiInit {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiDeinit;

#[derive(Format)]
pub struct RpcRespWifiDeinit {
    pub resp: i32,
}

// ---------- WiFi Config ----------
// #[derive(Format)]
pub struct RpcReqWifiSetConfig {
    pub iface: i32,
    pub cfg: WifiConfig,
}

#[derive(Format)]
pub struct RpcRespWifiSetConfig {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiGetConfig {
    pub iface: i32,
}

// #[derive(Format)]
pub struct RpcRespWifiGetConfig {
    pub resp: i32,
    pub iface: i32,
    pub cfg: WifiConfig,
}

// ---------- WiFi Control ----------
#[derive(Format)]
pub struct RpcReqWifiConnect;

#[derive(Format)]
pub struct RpcRespWifiConnect {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiDisconnect;

#[derive(Format)]
pub struct RpcRespWifiDisconnect {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiStart;

#[derive(Format)]
pub struct RpcRespWifiStart {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiStop;

#[derive(Format)]
pub struct RpcRespWifiStop {
    pub resp: i32,
}

// ---------- WiFi Scanning ----------
// #[derive(Format)]
pub struct RpcReqWifiScanStart {
    pub config: WifiScanConfig,
    pub block: bool,
    pub config_set: i32,
}

// impl RpcReqWifiScanStart {
//     impl RpcReqWifiScanStart {
//         pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
//             let mut i = 0;
//
//             write_rpc(buf, 1, Len, config_len as u64, &mut i);
//
//             i
//         }
//     }
// }

#[derive(Format)]
pub struct RpcRespWifiScanStart {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiScanStop;

#[derive(Format)]
pub struct RpcRespWifiScanStop {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiScanGetApNum;

#[derive(Format)]
pub struct RpcRespWifiScanGetApNum {
    pub resp: i32,
    pub number: i32,
}

#[derive(Format)]
pub struct RpcReqWifiScanGetApRecords {
    pub number: i32,
}

// #[derive(Format)]
pub struct RpcRespWifiScanGetApRecords {
    pub resp: i32,
    pub number: i32,
    pub ap_records: Vec<WifiApRecord, 50>,
}

#[derive(Format)]
pub struct RpcReqWifiScanGetApRecord;

// #[derive(Format)]
pub struct RpcRespWifiScanGetApRecord {
    pub resp: i32,
    pub ap_record: WifiApRecord,
}

// ---------- WiFi List Management ----------
#[derive(Format)]
pub struct RpcReqWifiClearApList;

#[derive(Format)]
pub struct RpcRespWifiClearApList {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiRestore;

#[derive(Format)]
pub struct RpcRespWifiRestore {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiClearFastConnect;

#[derive(Format)]
pub struct RpcRespWifiClearFastConnect {
    pub resp: i32,
}

// ---------- WiFi Deauth & STA ----------
#[derive(Format)]
pub struct RpcReqWifiDeauthSta {
    pub aid: i32,
}

#[derive(Format)]
pub struct RpcRespWifiDeauthSta {
    pub resp: i32,
    pub aid: i32,
}

#[derive(Format)]
pub struct RpcReqWifiStaGetApInfo;

// #[derive(Format)]
pub struct RpcRespWifiStaGetApInfo {
    pub resp: i32,
    pub ap_record: WifiApRecord,
}

// ---------- WiFi Protocol/Bandwidth/Channel ----------
#[derive(Format)]
pub struct RpcReqWifiSetProtocol {
    pub ifx: i32,
    pub protocol_bitmap: i32,
}

#[derive(Format)]
pub struct RpcRespWifiSetProtocol {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiGetProtocol {
    pub ifx: i32,
}

#[derive(Format)]
pub struct RpcRespWifiGetProtocol {
    pub resp: i32,
    pub protocol_bitmap: i32,
}

#[derive(Format)]
pub struct RpcReqWifiSetBandwidth {
    pub ifx: i32,
    pub bw: i32,
}

#[derive(Format)]
pub struct RpcRespWifiSetBandwidth {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiGetBandwidth {
    pub ifx: i32,
}

#[derive(Format)]
pub struct RpcRespWifiGetBandwidth {
    pub resp: i32,
    pub bw: i32,
}

#[derive(Format)]
pub struct RpcReqWifiSetChannel {
    pub primary: i32,
    pub second: i32,
}

#[derive(Format)]
pub struct RpcRespWifiSetChannel {
    pub resp: i32,
}

#[derive(Format)]
pub struct RpcReqWifiGetChannel;

#[derive(Format)]
pub struct RpcRespWifiGetChannel {
    pub resp: i32,
    pub primary: i32,
    pub second: i32,
}
