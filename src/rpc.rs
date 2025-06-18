//! Details about the RPC part of the protocol; this is how our payload
//! is packaged into the Payload header and TLV structure.

use defmt::{Format, println};
use num_enum::TryFromPrimitive;

use crate::{
    DataError, copy_le,
    protocol::{HEADER_SIZE, RPC_HEADER_MAX_SIZE, build_frame},
    rpc_enums::{PayloadCase, RpcId},
    transport::{RPC_EP_NAME_EVT, RPC_EP_NAME_RSP},
    util::{decode_varint, encode_varint},
};
use crate::util::write_rpc_var;

const MAX_RPC_SIZE: usize = 100; // todo temp.

/// Used in a few places when setting up RPC.
pub(crate) fn make_tag(field: u8, wire_type: WireType) -> u8 {
    (field << 3) | (wire_type as u8)
}

// todo: Experimenting.
/// esp_hosted_rpc.pb-c.h
pub(crate) struct Rpc {
    /// E.g. send a request, and receive a response or event. Varint.
    pub msg_type: RpcType,
    /// Identifies the type of message we're sending.
    pub msg_id: RpcId, // Encoded as varint.
    /// This is used for tracking purposes; it can be anything we want. Responses will match it. Varint.
    pub uid: u32,
}

impl Rpc {
    pub fn new_req(msg_id: RpcId, uid: u32) -> Self {
        Self {
            msg_type: RpcType::Req,
            msg_id,
            uid,
        }
    }

    /// Returns the total size used.
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        let mut i = 0;

        // todo: Is this the right wire type? (All fields)
        write_rpc_var(buf, 1, WireType::Basic, self.msg_type as u64, &mut i);
        write_rpc_var(buf, 2, WireType::Basic, self.msg_id as u64, &mut i);
        // write_rpc_var(buf, 3, WireType::Basic, self.uid as u64, &mut i);
        write_rpc_var(buf, 3, WireType::LenDetermined, self.uid as u64, &mut i);

        i
    }
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

        // todo: experimenting...
        //// -----------
        let rpc = Rpc::new_req(RpcId::ReqWifiApGetStaList, 0);
        return i; // todo...

        //----------

        // Field 1: id
        buf[i] = make_tag(1, WireType::Basic);
        i += 1;

        i += encode_varint(self.id as u64, &mut buf[i..]);

        // todo: It appears we must skip this in at least some cases. Fixed-len only, or always?
        // Field 2: payload_len
        // buf[i] = make_tag(2, WireType::Basic);
        // i += 1;
        //
        // i += encode_varint(self.len as u64, &mut buf[i..]);

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

        // if buf[i] != make_tag(2, WireType::Basic) {
        if buf[i] != make_tag(2, WireType::LenDetermined) {
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
/// See `esp_hosted_rpc.proto`, enum by this name. And `esp_hosted_rpc.pb-c.h`. (Maybe taken from there?)
/// We encode this as a varint.
pub(crate) enum RpcType {
    MsgType_Invalid = 0,
    Req = 1,
    Resp = 2,
    Event = 3,
    MsgType_Max = 4,
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

/// Sets up an RPC command. This makes calls to set up payload header and TLV.
/// returns the total payload size after setup. (Including PL header, TLV, RPC).
pub fn setup_rpc2(frame: &mut [u8], rpc_hdr: &RpcHeader, data: &[u8]) -> usize {
    // todo: I don't like how we need to specify another size constraint here, in addition to the frame buf.
    let mut rpc_buf = [0; MAX_RPC_SIZE];

    let data_len = data.len();

    // We use a separate buffer for the RPC header, since we must encode its size
    // prior to its contents, but its size comes about after we learn its contents.
    let mut rpc_hdr_buf = [0; RPC_HEADER_MAX_SIZE];

    let hdr_len = rpc_hdr.to_bytes(&mut rpc_hdr_buf);
    // println!("Header len: {:?}", hdr_len);

    // This `i` is relative to the RPC section, after the payload header.
    let mut i = 0;

    // todo: What should these Wire Types be??

    // RPC header len, and its tag.
    // rpc_buf[i] = make_tag(1, WireType::Basic);
    // i += 1;
    // i += encode_varint(hdr_len as u64, &mut rpc_buf[i..]);

    println!("header buf: {:?}", &rpc_hdr_buf);
    println!("rpc buf: {:?}", &rpc_buf);

    // RPC header
    rpc_buf[i..i + hdr_len].copy_from_slice(&rpc_hdr_buf[..hdr_len]);
    i += hdr_len;

    // Data Len, and its tag.
    // rpc_buf[i] = make_tag(2, WireType::Basic);
    rpc_buf[i] = make_tag(2, WireType::LenDetermined);
    i += 1;
    i += encode_varint(data_len as u64, &mut rpc_buf[i..]);

    rpc_buf[i..i + data_len].copy_from_slice(data);
    i += data_len;

    build_frame(frame, &rpc_buf[..i])
}

/// Sets up an RPC command. This makes calls to set up payload header and TLV.
/// returns the total payload size after setup. (Including PL header, TLV, RPC).
pub fn setup_rpc(frame: &mut [u8], rpc_hdr: &RpcHeader, data: &[u8]) -> usize {
    // todo: I don't like how we need to specify another size constraint here, in addition to the frame buf.
    let mut rpc_buf = [0; MAX_RPC_SIZE];


    // We use a separate buffer for the RPC header, since we must encode its size
    // prior to its contents, but its size comes about after we learn its contents.
    // let mut rpc_hdr_buf = [0; RPC_HEADER_MAX_SIZE];

    let mut i = 0;
    // let hdr_len = rpc_hdr.to_bytes(&mut rpc_buf);
    // i += hdr_len;

    let rpc = Rpc::new_req(RpcId::ReqWifiApGetStaList, 0);
    i += rpc.to_bytes(&mut rpc_buf);

    println!("RPC len pre-data: {}", i);

    let data_len = data.len();
    rpc_buf[i..i + data_len].copy_from_slice(data);
    i += data_len;

    println!("RPC buf: {:?} - Len: {}", &rpc_buf[..i], i);

    build_frame(frame, &rpc_buf[..i])
}