//! Details about the RPC part of the protocol; this is how our payload
//! is packaged into the Payload header and TLV structure.

use defmt::{Format, println};
use num_enum::TryFromPrimitive;

use crate::{
    protocol::build_frame,
    rpc_enums::RpcId,
    transport::{RPC_EP_NAME_EVT, RPC_EP_NAME_RSP},
    util::encode_varint,
};

const MAX_RPC_SIZE: usize = 100; // todo temp.

/// Used in a few places when setting up RPC.
// pub(crate) fn make_tag(field: u16, wire_type: WireType) -> u8 {
pub(crate) fn make_tag(field: u16, wire_type: WireType) -> u16 {
    (field << 3) | (wire_type as u16)
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

    /// Writes the Rpc struct and data to buf. Returns the total size used.
    pub fn to_bytes(&self, buf: &mut [u8], data: &[u8]) -> usize {
        let mut i = 0;
        let data_len = data.len();

        // todo: Is this the right wire type? (All fields)
        write_rpc_var(buf, 1, WireType::Basic, self.msg_type as u64, &mut i);
        write_rpc_var(buf, 2, WireType::Basic, self.msg_id as u64, &mut i);
        write_rpc_var(buf, 3, WireType::Basic, self.uid as u64, &mut i);

        // We repeat the message id as the payload's tag field.
        // Note: When using length-determined, we must follow the tag with a varint len.
        write_rpc_var(
            buf,
            self.msg_id as u16,
            WireType::LenDetermined,
            data_len as u64,
            &mut i,
        );

        buf[i..i + data_len].copy_from_slice(data);
        i += data_len;

        i


        // Buf: `[8, 1, 16, 183, 2, 24, 0, || 186, 19, 0]`
        // 8: Tag(Field 1, wire type: basic)
        // 1: Message type request
        // 16: Tag (Field 2, wire type: basic)
        // 183, 2: Msg ID (RPC ID) 311: ReqWifiApGetStaList
        // 24: Tag (Field 3, wire type: basic)

        // 186, 19, 0 // "empty message??"
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
