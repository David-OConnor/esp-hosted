//! This module contains the most important parts of the protocol.

use defmt::Format;
use num_enum::TryFromPrimitive;

use crate::{copy_le, parse_le, transport::compute_checksum};
use crate::transport::{RPC_EP_NAME_EVT, RPC_EP_NAME_RSP};
// pub(crate) const MAGIC: u8 = 0xEC;
// pub(crate) const VERSION: u8 = 1;

pub(crate) const PL_HEADER_SIZE: usize = 12; // Verified from ESP docs

// The 6 static bytes in the TLV header: endpoint type (1), endpoint length (2), data type (1),
// data length (2). Doesn't include the endpoint value, which is 8-10 (?)
const TLV_HEADER_SIZE: usize = 6;
// RPC_EP_NAME_EVT is the same size.
pub(crate) const TLV_SIZE: usize = TLV_HEADER_SIZE + RPC_EP_NAME_RSP.len();

// Contains both the PL header and TLV header. Everything except the payload.
pub(crate) const HEADER_SIZE: usize = PL_HEADER_SIZE + TLV_SIZE;


#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u8)]
// See `esp_hosted_rpc.proto`, enum by this name.
pub(crate) enum Rpc_WifiBw {
    BW_Invalid = 0,
    HT20 = 1,
    HT40 = 2,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u8)]
// See `esp_hosted_rpc.proto`, enum by this name.
pub(crate) enum Rpc_WifiPowerSave {
    PS_Invalid = 0,
    MIN_MODEM = 1,
    MAX_MODEM = 2,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u8)]
// See `esp_hosted_rpc.proto`, enum by this name.
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
// See `esp_hosted_rpc.proto`, enum by this name.
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
// See `esp_hosted_rpc.proto`, enum by this name.
pub(crate) enum RpcType {
    MsgType_Invalid = 0,
    Req = 1,
    Resp = 2,
    Event = 3,
    MsgType_Max = 4,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u16)]
// See `esp_hosted_rpc.proto`, enum by this name.
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
pub(crate) enum PacketTypeHci {
    A = 0,
    B = 1, // todo
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
/// Private
pub(crate) enum PacketTypePriv {
    A = 0,
    B = 1, // todo
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PacketType {
    // todo??
    Hci(PacketTypeHci),
    Priv(PacketTypePriv),
    // /* data-path kinds — rarely used on the control UART */
    // WifiData = 0x00, // 802.3 / 802.11 data via netif
    // // BtHci      = 0x01,  // HCI ACL / EVT
    // // todo: Oh my. What actually are these?
    // HciCommand = 0x01,
    // HciAclData = 0x02,
    // HciScoData = 0x03,
    // HciEvent = 0x04,
    // HciIsoData = 0x05,
    // // SerialPty  = 0x02,  // AT/PTY stream
    // /* … other data codes, e.g. SCO = 0x03 … */
    // /* private / control events */
    // Event = 0x33, // ← what goes in Ping/Pong, scan results, etc.
}

impl PacketType {
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::Hci(p) => *p as u8,
            Self::Priv(p) => *p as u8,
        }
    }

    pub fn from_byte(val: u8) -> Self {
        // todo temp; fix.
        Self::Priv(PacketTypePriv::A)
    }
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
/// Interface type. See ESP-Hosted-MCU readme, section 7.2
/// /// [official enum](https://github.com/espressif/esp-hosted-mcu/blob/634e51233af2f8124dfa8118747f97f8615ea4a6/common/esp_hosted_interface.h)
pub(crate) enum IfType {
    Invalid = 0,
    Sta = 1,
    Ap = 2,
    Serial = 3,
    Hci = 4,
    Priv = 5,
    Test = 6,
    Eth = 7,
    Max = 8,
}

#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
// todo: Verify the provenance.
pub(crate) enum Module {
    /// “system / housekeeping”
    Ctrl = 0x00,
    Wifi = 0x01,
    Ble = 0x02,
}

// todo: DO I need this SLIP-encoding? UART-only, if so.
/// SLIP-encode into `out`, return number of bytes written.
/// // todo: Verify the provenance.
pub(crate) fn slip_encode(src: &[u8], out: &mut [u8]) -> usize {
    const END: u8 = 0xC0;
    const ESC: u8 = 0xDB;
    const ESC_END: u8 = 0xDC;
    const ESC_ESC: u8 = 0xDD;

    let mut w = 0;
    out[w] = END; // flush garbage on the line
    w += 1;

    for &b in src {
        match b {
            END => {
                out[w] = ESC;
                out[w + 1] = ESC_END;
                w += 2;
            }
            ESC => {
                out[w] = ESC;
                out[w + 1] = ESC_ESC;
                w += 2;
            }
            _ => {
                out[w] = b;
                w += 1;
            }
        }
    }
    out[w] = END;
    w + 1 // trailing END
}

/// Adapted from `esp-hosted-mcu/common/esp_hosted_header.h`
/// This is at the start of the message, and is followed by the RPC header.
/// See ESP-hosted-MCU readme, section 7.1.
struct PayloadHeader {
    /// Interface type
    pub if_type: IfType, // 2 4-bit values
    /// Interface number
    pub if_num: u8, // 2 4-bit values
    pub flags: u8,
    /// Payload length
    /// "Payload starts the next byte after header->offset"
    pub len: u16,
    /// Offset to payload. todo: Is this always 12?
    pub offset: u16,
    /// Header + payload checksum
    pub checksum: u16,
    /// Sequence number for tracking packets (Useful in debugging)
    pub seq_num: u16,
    /// Flow control
    pub throttle_cmd: u8, // First two bits of this byte.
    // u8 reserved; `reserve2:6`; same byte as `throttle_cmd`

    // From Esp doc: First 3 bits may be reserved. The remaining bits for HCI or PRivate packet type?
    pub pkt_type: PacketType,
}

impl PayloadHeader {
    pub fn new(if_type: IfType, pkt_type: PacketType,  payload_len: usize) -> Self {
        // Len is the number of bytes following the header. (all)
        let len = (TLV_SIZE + payload_len) as u16;

        Self {
            if_type,
            if_num: 0,
            flags: 0,
            len,
            offset: PL_HEADER_SIZE as u16,
            // Computed after the entire frame is constructed. Must be set to 0 for
            // now, as this goes into the checksum calculation.
            checksum: 0,
            // Todo: This should increment.
            seq_num: 0,
            throttle_cmd: 0,
            pkt_type,
        }
    }
    /// Serialize into the 12-byte packed representation
    pub fn to_bytes(&self) -> [u8; PL_HEADER_SIZE] {
        let mut buf = [0; PL_HEADER_SIZE];

        // byte 0:   [ if_num:4 | if_type:4 ]
        buf[0] = (self.if_num << 4) | ((self.if_type as u8) & 0x0F);

        buf[1] = self.flags;

        copy_le!(buf, self.len, 2..4);
        copy_le!(buf, self.offset, 4..6);
        copy_le!(buf, self.checksum, 6..8);
        copy_le!(buf, self.seq_num, 8..10);

        // byte 10:  [ reserved2:6 | throttle_cmd:2 ]
        buf[10] = self.throttle_cmd; // todo: QC if you need a shift.

        // byte 11: union field
        buf[11] = self.pkt_type.to_byte();

        buf
    }

    /// Parse from a 12-byte slice (will panic if `buf.len() < 12` or slice-to-array fails)
    pub fn from_bytes(buf: &[u8]) -> Self {
        let if_type = (buf[0] & 0x0F).try_into().unwrap();
        let if_num = (buf[0] >> 4) & 0x0F;
        let flags = buf[1];

        let len = parse_le!(buf, u16, 2..4);
        let offset = parse_le!(buf, u16, 4..6);
        let checksum = parse_le!(buf, u16, 6..8);
        let seq_num = parse_le!(buf, u16, 8..10);

        let throttle_cmd = buf[10] & 3;
        let pkt_type = PacketType::from_byte(buf[11]);

        Self {
            if_type,
            if_num,
            flags,
            len,
            offset,
            checksum,
            seq_num,
            throttle_cmd,
            pkt_type,
        }
    }
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
pub(crate) enum RpcEndpoint {
    CtrlResp,
    CtrlEvent,
}

impl RpcEndpoint {
    pub fn as_bytes(&self) -> &'static [u8] {
         match self {
            Self::CtrlResp   => RPC_EP_NAME_RSP,
            Self::CtrlEvent  => RPC_EP_NAME_EVT
        }.as_bytes()
    }
    // pub fn to_bytes(&self, buf: &mut [u8]) {
    //     let slice: &[u8] = match self {
    //         Self::CtrlResp   => RPC_EP_NAME_RSP,
    //         Self::CtrlEvent  => RPC_EP_NAME_EVT
    //     }.as_bytes();
    //     buf[0..slice.len()].copy_from_slice(slice);
    // }
    //
    // pub fn endpoint_length(&self) -> u16 {
    //     match self {
    //         Self::CtrlResp => RPC_EP_NAME_RSP.len() as u16,
    //         Self::CtrlEvent => 10,
    //     }
    // }
}

// /// Type-length-value header. See host/drivers/virtual_serial_if/serial_if.c
// struct TlvHeader {
//     // pub endpoint_type: TlvType,
//     pub endpoint_length: u16,
//     pub endpoint_value: EndpointValue,
//     pub data_length: u16,
//     // pub data_value: &[u8], // data length
// }

/// Frame structure:
/// Bytes 0..12: Payload header.
/// Bytes 12..18: Payload
/// Bytes 18..18 + payload len: Payload
/// Trailing 2 bytes: CRC (frame; different from the payload header CRC)
// pub(crate) fn build_frame(out: &mut [u8], rpc_mod: Module, rpc_cmd: Command, payload: &[u8]) {
pub(crate) fn build_frame(
    out: &mut [u8],
    // endpoint_type: EndpointType,
    // data_type: EndpointType,
    // endpoint: &[u8],
    // todo: A message type here likely.
    payload: &[u8],
) {
    let payload_len = payload.len();

    // From `serial_if.c`: Always Resp for compose. Either Resp or Event from parse. (host-side)
    let endpoint_value = RpcEndpoint::CtrlResp.as_bytes();
    let endpoint_len = endpoint_value.len() as u16;

    let payload_header = PayloadHeader::new(
        IfType::Serial,
        PacketType::Priv(PacketTypePriv::A),
        payload_len,
    );
    out[..PL_HEADER_SIZE].copy_from_slice(&payload_header.to_bytes());

    let mut i = PL_HEADER_SIZE;

    out[i] = EndpointType::EndpointName as u8;
    i += 1;

    copy_le!(out, endpoint_len, i..i + 2);
    i += 2;

    out[i..i + endpoint_len as usize].copy_from_slice(endpoint_value);
    i += endpoint_len as usize;

    out[i] = EndpointType::Data as u8;
    i += 1;

    copy_le!(out, payload_len as u16, i..i + 2);
    i += 2;

    out[i..i + payload_len].copy_from_slice(payload);
    i += payload_len;

    // system_design...: "**Checksum Coverage**: The checksum covers the **entire frame** including:
    // 1. Complete `esp_payload_header` (with checksum field set to 0 during calculation)
    // 2. Complete payload data"
    let pl_checksum = compute_checksum(&out[..i]);
    defmt::println!("Pl checksum we're sending: {:?}", pl_checksum);
    copy_le!(out, pl_checksum, 6..8);
}
