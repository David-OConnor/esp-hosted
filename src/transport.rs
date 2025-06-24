//! From `esp_hosted_transport.h`

use defmt::Format;
use num_enum::TryFromPrimitive;

use crate::{EspError, HciPkt};

const PRIO_Q_SERIAL: u8 = 0;
const PRIO_Q_BT: u8 = 1;
const PRIO_Q_OTHERS: u8 = 2;
const MAX_PRIORITY_QUEUES: u8 = 3;
const MAC_SIZE_BYTES: u8 = 6;

/* Serial interface */
const SERIAL_IF_FILE: &str = "/dev/esps0";

/* Protobuf related info */
/* Endpoints registered must have same string length */
pub(crate) const RPC_EP_NAME_RSP: &str = "RPCRsp";
pub(crate) const RPC_EP_NAME_EVT: &str = "RPCEvt";

#[derive(Clone, Copy, PartialEq, Default, Format)]
pub(crate) enum PacketType {
    #[default]
    None,
    /// It appears that this is only used from the Slave
    ESP_PACKET_TYPE_EVENT,
    /// It appears that this is always the type sent by the host.
    ESP_PRIV_EVENT_INIT,
    Hci(HciPkt),
}

impl PacketType {
    pub fn val(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::ESP_PACKET_TYPE_EVENT => 0x33,
            Self::ESP_PRIV_EVENT_INIT => 0x22,
            Self::Hci(v) => *v as u8,
        }
    }

    pub fn from_byte(b: u8) -> Result<Self, EspError> {
        Ok(match b {
            0 => Self::None,
            0x33 => Self::ESP_PACKET_TYPE_EVENT,
            0x22 => Self::ESP_PACKET_TYPE_EVENT,
            _ => Self::Hci(HciPkt::try_from(b).map_err(|_| EspError::InvalidData)?),
        })
    }
}

#[repr(u8)]
pub(crate) enum SLAVE_CONFIG_PRIV_TAG_TYPE {
    HOST_CAPABILITIES = 0x44,
    RCVD_ESP_FIRMWARE_CHIP_ID,
    SLV_CONFIG_TEST_RAW_TP,
    SLV_CONFIG_THROTTLE_HIGH_THRESHOLD,
    SLV_CONFIG_THROTTLE_LOW_THRESHOLD,
}

pub(crate) const ESP_TRANSPORT_SDIO_MAX_BUF_SIZE: u16 = 1536;
pub(crate) const ESP_TRANSPORT_SPI_MAX_BUF_SIZE: u16 = 1600;
pub(crate) const ESP_TRANSPORT_SPI_HD_MAX_BUF_SIZE: u16 = 1600;
pub(crate) const ESP_TRANSPORT_UART_MAX_BUF_SIZE: u16 = 1600;

pub(crate) struct esp_priv_event {
    event_type: u8,
    event_len: u8,
    event_data: u8, // ([0]??) Is this an arary?
}

/// `System_design_with_rps_as_focus.md`, section 3.3: Checksum Calculation
pub(crate) fn compute_checksum(buf: &[u8]) -> u16 {
    let mut checksum = 0;
    let mut i = 0;

    while i < buf.len() {
        checksum += buf[i] as u16;
        i += 1;
    }

    checksum
}
