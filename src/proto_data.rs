#![allow(unused)]

//! Contains data types specific to the ESP-Hosted-MCU proto buffer. This contains
//! definitions for the data types [de]serialized. They're not automatically generated
//! from the .proto file, and are used in our higher-level API.

use defmt::{Format, println};
use heapless::Vec;
use num_enum::TryFromPrimitive;

use crate::{
    rpc::{
        WireType,
        WireType::{Len, Varint},
        write_rpc,
    },
    wifi::{InitConfig, ScanConfig},
};

const MAX_DATA_SIZE: usize = 300; // todo temp

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
    /// In seconds. Min of 10.
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
pub struct EventHeartbeat {
    /// Number of beats
    pub number: u32,
}

// ---------- WiFi Init/Deinit ----------
#[derive(Format)]
pub struct RpcReqWifiInit {
    pub cfg: InitConfig,
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

// #[derive(Format)]
pub struct RpcRespWifiGetConfig {
    pub resp: i32,
    pub iface: i32,
    pub cfg: WifiConfig,
}

// #[derive(Default, Format)]
#[derive(Default)]
pub struct RpcReqWifiScanStart {
    pub config: ScanConfig,
    /// true → RPC blocks until scan complete; false → returns immediately and you wait for
    /// WIFI_SCAN_DONE then pull the list.
    pub block: bool,
    /// Bit-mask telling the firmware how much of config you’re sending. 1 means “use everything
    /// in config once”. Leave 0 if you previously uploaded a config and just want to trigger another scan.
    pub config_set: i32,
}

impl RpcReqWifiScanStart {
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        let mut i = 0;

        let mut cfg_buf = [0; 50]; // ~22 with defaults.
        let cfg_len = self.config.to_bytes(&mut cfg_buf);

        println!("Scan cfg len: {:?}", cfg_len); // todo: Use to set buf appropriately.

        write_rpc(buf, 1, Len, cfg_len as u64, &mut i);
        buf[i..i + cfg_len].copy_from_slice(&cfg_buf[..cfg_len]);
        i += cfg_len;

        write_rpc(buf, 2, Varint, self.block as u64, &mut i);
        write_rpc(buf, 3, Varint, self.config_set as u64, &mut i);

        i
    }
}

#[derive(Format)]
pub struct RpcReqWifiSetChannel {
    pub primary: i32,
    pub second: i32,
}

#[derive(Format)]
pub struct RpcRespWifiGetChannel {
    pub primary: i32,
    pub second: i32,
}
