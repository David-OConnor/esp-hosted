//! Details about the RPC part of the protocol; this is how our payload
//! is packaged into the Payload header and TLV structure.

use defmt::{println, Format};
use num_enum::TryFromPrimitive;
use crate::{copy_le, DataError};
use crate::protocol::{build_frame, RPC_HEADER_MAX_SIZE};
use crate::transport::{RPC_EP_NAME_EVT, RPC_EP_NAME_RSP};
use crate::util::{decode_varint, encode_varint};

/// Used in a few places when setting up RPC.
pub(crate) fn make_tag(field: u8, wire_type: WireType) -> u8 {
    (field <<3) | (wire_type as u8)
}

/// Wire types:
#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum WireType {
    /// 0: int32, uint32, bool, enum, variant
    Basic = 0,
    /// 64-bit fixed
    SixtyFourBit = 1,
    /// Len-determined (string, bytes, sub-message)
    LenDetermined = 2,
    /// 32-bit fixed (fixed32, float)
    Fixed32Bit = 5,
}

pub(crate) struct RpcHeader {
    pub id: RpcId,
    pub len: u16,
}

impl RpcHeader {
    /// Encode as protobuf (wire-type 0) into `buf`.
    /// Returns the number of bytes written.
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        let mut i = 0;

        // Field 1: id
        buf[i] = make_tag(1, WireType::Basic);
        i += 1;
        i += encode_varint(self.id as u64, &mut buf[i..]);

        // Field 2: payload_len
        buf[i] = make_tag(2, WireType::Basic);
        i += 1;
        i += encode_varint(self.len as u64, &mut buf[i..]);

        i
    }

    /// Decode from `buf`, returning `(header, bytes_consumed)`.
    pub fn from_bytes(buf: &[u8]) -> Result<(Self, usize), DataError> {
        let mut i = 0;

        if buf[i] != make_tag(1, WireType::Basic) {
            return Err(DataError::Invalid);
        }

        i += 1;
        let (id_val, n) = decode_varint(&buf[i..]);
        i += n;

        if buf[i] != make_tag(2, WireType::Basic) {
            return Err(DataError::Invalid);
        }

        i += 1;
        let (len_val, n) = decode_varint(&buf[i..]);
        i += n;

        Ok((
            Self {
                id: (id_val as u16).try_into().unwrap(),
                len: len_val as u16,
            },
            i,
        ))
    }
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u8)]
/// See `esp_hosted_rpc.proto`, enum by this name.
pub(crate) enum Rpc_WifiBw {
    BW_Invalid = 0,
    HT20 = 1,
    HT40 = 2,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u8)]
/// See `esp_hosted_rpc.proto`, enum by this name.
pub(crate) enum Rpc_WifiPowerSave {
    PS_Invalid = 0,
    MIN_MODEM = 1,
    MAX_MODEM = 2,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u8)]
/// See `esp_hosted_rpc.proto`, enum by this name.
pub(crate) enum Rpc_WifiSecProt {
    Open = 0,
    WEP = 1,
    WPA_PSK = 2,
    WPA2_PSK = 3,
    WPA_WPA2_PSK = 4,
    WPA2_ENTERPRISE = 5,
    WPA3_PSK = 6,
    WPA2_WPA3_PSK = 7,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u8)]
/// See `esp_hosted_rpc.proto`, enum by this name.
pub(crate) enum Rpc_Status {
    Connected = 0,
    Not_Connected = 1,
    No_AP_Found = 2,
    Connection_Fail = 3,
    Invalid_Argument = 4,
    Out_Of_Range = 5,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u8)]
/// See `esp_hosted_rpc.proto`, enum by this name.
pub(crate) enum RpcType {
    MsgType_Invalid = 0,
    Req = 1,
    Resp = 2,
    Event = 3,
    MsgType_Max = 4,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u16)]
/// See `esp_hosted_rpc.proto`, enum by this name.
pub(crate) enum RpcId {
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

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
/// Type-length-value header. See host/drivers/virtual_serial_if/serial_if.c
pub(crate) enum EndpointType {
    /// PROTO_PSER_TLV_T_EPNAME
    EndpointName = 0x01,
    /// PROTO_PSER_TLV_T_DATA
    Data = 0x02,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
/// Type-length-value header. See host/drivers/virtual_serial_if/serial_if.c
/// Note that other sources imply there should also be a CtrlReq, which is the only
/// one the host sends. (Is that from esp-hosted-nonmcu?)
pub(crate) enum RpcEndpoint {
    CtrlResp,
    CtrlEvent,
}

impl RpcEndpoint {
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            /// Host only sends this.
            Self::CtrlResp => RPC_EP_NAME_RSP,
            /// Slave sends either.
            Self::CtrlEvent => RPC_EP_NAME_EVT,
        }
            .as_bytes()
    }
}


/// Call this function to set up an RPC command. This makes calls to set up payload header and TLV.
/// returns the total payload size after setup. (Including PL header, TLV, RPC)
pub fn setup_rpc(frame: &mut [u8], rpc_hdr: &RpcHeader, data: &[u8]) -> usize {
    // We must allocate this buffer as fixed size; 9 will do for here.

    let mut hdr_buf = [0u8; RPC_HEADER_MAX_SIZE];
    let hdr_len = rpc_hdr.to_bytes(&mut hdr_buf);

    let mut payload = [0; RPC_HEADER_MAX_SIZE + 5];
    let mut i = 0;


    payload[i] = make_tag(1, WireType::LenDetermined);
    i += 1;
    i += encode_varint(hdr_len as u64, &mut payload[i..]);
    payload[i..i + hdr_len].copy_from_slice(&hdr_buf[..hdr_len]);
    i += hdr_len;

    payload[i] = make_tag(2, WireType::LenDetermined);
    i += 1;
    i += encode_varint(data.len() as u64, &mut payload[i..]);
    payload[i..i + data.len()].copy_from_slice(data);
    i += data.len();

    let frame_size = build_frame(frame, &payload[..i]);

    // todo: Do we add a trailing checksum? ChatGPT thinks so half the time, but I can't find
    // todo evidence anywhere for it.
    let checksum_trail = 0;
    // copy_le!(frame, checksum_trail, i..i+2);
    // i += 2;

    frame_size
}