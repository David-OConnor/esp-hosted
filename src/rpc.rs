//! Details about the RPC part of the protocol; this is how our payload
//! is packaged into the Payload header and TLV structure. It contains a mix of protobuf-general
//! concepts, and ones specific to the RPC used by esp-hosted-mcu.

use defmt::{Format, println};
use num_enum::TryFromPrimitive;

use crate::{
    EspError,
    header::build_frame,
    proto_data::RpcId,
    transport::{RPC_EP_NAME_EVT, RPC_EP_NAME_RSP},
};

const MAX_RPC_SIZE: usize = 100; // todo temp.
pub(crate) const RPC_MIN_SIZE: usize = 10;


/// See `esp_hosted_rpc.proto`.
#[derive(Format)]
pub struct Rpc {
    /// E.g. send a request, and receive a response or event.
    pub msg_type: RpcType,
    /// Identifies the type of message we're sending.
    pub msg_id: RpcId,
    /// This is used for tracking purposes; it can be anything we want. Responses will match it.
    pub uid: u32,
    // This is followed by a (len-determined; sub-struct) field associated with the RPC Id.
    // It has field number = Rpc ID.
}

impl Rpc {
    pub fn new_req(msg_id: RpcId, uid: u32) -> Self {
        Self {
            msg_type: RpcType::Req,
            msg_id,
            uid,
        }
    }

    /// Writes the Rpc struct and data to buf. Returns the total size used.
    pub fn to_bytes(&self, buf: &mut [u8], data: &[u8]) -> usize {
        let mut i = 0;
        let data_len = data.len();

        // todo: Is this the right wire type? (All fields)
        write_rpc_var(buf, 1, WireType::Varint, self.msg_type as u64, &mut i);
        write_rpc_var(buf, 2, WireType::Varint, self.msg_id as u64, &mut i);
        write_rpc_var(buf, 3, WireType::Varint, self.uid as u64, &mut i);

        // We repeat the message id as the payload's tag field.
        // Note: When using length-determined, we must follow the tag with a varint len.

        write_rpc_var(
            buf,
            self.msg_id as u16,
            WireType::Len,
            data_len as u64,
            &mut i,
        );

        buf[i..i + data_len].copy_from_slice(data);
        i += data_len;

        i
    }

    /// Returns (Self, data start i, data len expected).
    pub fn from_bytes(buf: &[u8]) -> Result<(Self, usize, usize), EspError> {
        let msg_type = buf[1].try_into().map_err(|e| EspError::InvalidData)?;

        let (rpc_id, rpc_id_size) = decode_varint(&buf[3..])?;
        let msg_id = (rpc_id as u16)
            .try_into()
            .map_err(|e| EspError::InvalidData)?;

        // We are skipping the UID tag; hence the jump from 3 to 4.
        let mut i = 4 + rpc_id_size;

        let (uid, uid_size) = decode_varint(&buf[i..])?;
        i += uid_size;

        let result = Self {
            msg_type,
            msg_id,
            uid: uid as u32,
        };


        let (_data_tag, data_tag_size) = decode_varint(&buf[i..])?;
        i += data_tag_size;

        let (data_len, data_len_size) = decode_varint(&buf[i..])?;
        i += data_len_size;

        Ok((result, i, data_len as usize))
    }
}

/// https://protobuf.dev/programming-guides/encoding/
#[derive(Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum WireType {
    /// i32, i64, u32, u64, sint64, sint32, sing64, bool, enum
    Varint = 0,
    /// fixed64, sfixed64, double
    I64 = 1,
    /// Len-determined (string, bytes, embedded messages, packed repeated fields)
    Len = 2,
    /// 32-bit fixed (fixed32, sfixed32, float)
    I32 = 5,
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
pub enum RpcType {
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
// pub fn setup_rpc(frame: &mut [u8], rpc_hdr: &RpcHeader, data: &[u8]) -> usize {
pub fn setup_rpc(frame: &mut [u8], rpc: &Rpc, data: &[u8]) -> usize {
    // todo: I don't like how we need to specify another size constraint here, in addition to the frame buf.
    let mut rpc_buf = [0; MAX_RPC_SIZE];

    let mut i = 0;

    i += rpc.to_bytes(&mut rpc_buf, data);

    println!("RPC len: {}", i);
    println!("RPC buf: {:?}", &rpc_buf[..i]);

    build_frame(frame, &rpc_buf[..i])
}

/// Handles making tags, and encoding as varints. Increments the index.
/// todo: Consider here, and `encode_varint`, using u32 vice u64.
pub(crate) fn write_rpc_var(
    buf: &mut [u8],
    field: u16,
    wire_type: WireType,
    val: u64,
    i: &mut usize,
) {
    let tag = make_tag(field, wire_type);
    *i += encode_varint(tag as u64, &mut buf[*i..]);
    *i += encode_varint(val, &mut buf[*i..]);
}


/// Used in a few places when setting up RPC.
pub(crate) fn make_tag(field: u16, wire_type: WireType) -> u16 {
    (field << 3) | (wire_type as u16)
}

/// Encodes `v` as little-endian 7-bit var-int.
/// Returns number of bytes written (1–3 for a `u16`).
pub(crate) fn encode_varint(mut v: u64, out: &mut [u8]) -> usize {
    let mut idx = 0;
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            out[idx] = byte; // last byte – high bit clear
            idx += 1;
            break;
        } else {
            out[idx] = byte | 0x80; // more bytes follow
            idx += 1;
        }
    }
    idx
}

/// Decodes a little-endian 7-bit var-int.
/// Returns `(value, bytes_consumed)`.
pub(crate) fn decode_varint(input: &[u8]) -> Result<(u64, usize), EspError> {
    let mut val = 0u64;
    let mut shift = 0;
    for (idx, &byte) in input.iter().enumerate() {
        val |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok((val, idx + 1));
        }
        shift += 7;
    }

    Err(EspError::InvalidData)
}
