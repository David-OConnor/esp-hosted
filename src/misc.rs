//! Adapted from C files in ESP-common. WIP

// Aadapted from `esp_hosted_transport.h`.

/* ---------- priority queues & generic constants ---------- */

/* ---------- transport MTU per physical bus ---------- */

pub const ESP_TRANSPORT_SDIO_MAX_BUF_SIZE: usize = 1536;
pub const ESP_TRANSPORT_SPI_MAX_BUF_SIZE: usize = 1600;
pub const ESP_TRANSPORT_SPI_HD_MAX_BUF_SIZE: usize = 1600;
pub const ESP_TRANSPORT_UART_MAX_BUF_SIZE: usize = 1600;



pub const PRIO_Q_SERIAL: u8 = 0;
pub const PRIO_Q_BT: u8 = 1;
pub const PRIO_Q_OTHERS: u8 = 2;
pub const MAX_PRIORITY_QUEUES: u8 = 3;

pub const MAC_SIZE_BYTES: usize = 6;

/* ---------- serial device ---------- */

pub const SERIAL_IF_FILE: &str = "/dev/esps0";

/* ---------- protobuf / RPC endpoints (same length!) ---------- */

pub const RPC_EP_NAME_RSP: &str = "RPCRsp";
pub const RPC_EP_NAME_EVT: &str = "RPCEvt";

/* ---------- host-side flow-control state ---------- */

#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum FlowCtrl {
    Nc = 0,  // “no change / unknown”
    On = 1,  // host permits ESP to send
    Off = 2, // host asks ESP to pause
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum SlaveConfigPrivTagType {
    HostCapabilities = 0x44,
    RcvdEspFirmwareChipId = 0x45,
    SlvConfigTestRawTp = 0x46,
    SlvConfigThrottleHighThreshold = 0x47,
    SlvConfigThrottleLowThreshold = 0x48,
}

/* ---------- packed event header (flex-array payload follows on the wire) ---------- */

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct PrivEventHeader {
    pub event_type: u8,
    pub event_len: u8, // length of `event_data`
    // `event_data` bytes follow immediately in the frame
}

/* ---------- checksum helper (identical semantics to C version) ---------- */


// Adapted from `esp_hosted_bitmasks.h`
// todo: Make into proper enums
// Bit-position and mask definitions that mirror
// `esp_hosted_bitmasks.h` (May 2025).
//
// • pure **`#![no_std]`**, no external crates
// • constants are `const` so the compiler can fold everything away
// • helpers are `#[inline(always)]` zero-cost

/* ------------------------------------------------------------------------- */
/*  small helpers – replacements for the C SET/GET “macros”                  */
/* ------------------------------------------------------------------------- */

/// Returns `val | (1 << pos)`
#[inline(always)]
pub const fn set_bit_u32(val: u32, pos: u8) -> u32 {
    val | (1u32 << pos)
}

/// Returns `((val >> pos) & 1) != 0`
#[inline(always)]
pub const fn get_bit_u32(val: u32, pos: u8) -> bool {
    ((val >> pos) & 1) != 0
}

/* ------------------------------------------------------------------------- */
/*  Wi-Fi **scan-AP-record** flag word                                       */
/* ------------------------------------------------------------------------- */

pub mod wifi_scan_ap_rec {
    /* bit positions (u16) */
    pub const phy_11b: u8 = 0;
    pub const phy_11g: u8 = 1;
    pub const phy_11n: u8 = 2;
    pub const phy_lr: u8 = 3;
    pub const phy_11ax: u8 = 4;
    pub const wps: u8 = 5;
    pub const ftm_responder: u8 = 6;
    pub const ftm_initiator: u8 = 7;
    pub const phy_11a: u8 = 8;
    pub const phy_11ac: u8 = 9;

    /// highest “used” bit index
    pub const MAX_USED_BIT: u8 = 10;

    /* masks --------------------------------------------------------------- */

    /// `0b1111_1100_0000_0000`
    pub const RESERVED_BITMASK: u16 = 0xFC00;

    /* helpers ------------------------------------------------------------- */

    /// Extract the *reserved* bits (already right-aligned).
    #[inline(always)]
    pub const fn get_reserved(num: u16) -> u16 {
        (num & RESERVED_BITMASK) >> MAX_USED_BIT
    }

    /// Overlay `reserved_in` into `num`.
    #[inline(always)]
    pub const fn set_reserved(num: u16, reserved_in: u16) -> u16 {
        num | (reserved_in << MAX_USED_BIT)
    }
}

/* ------------------------------------------------------------------------- */
/*  Wi-Fi **STA-info** flag word                                             */
/* ------------------------------------------------------------------------- */

pub mod wifi_sta_info {
    pub const phy_11b: u8 = 0;
    pub const phy_11g: u8 = 1;
    pub const phy_11n: u8 = 2;
    pub const phy_lr: u8 = 3;
    pub const phy_11ax: u8 = 4;
    pub const is_mesh_child: u8 = 5;

    pub const MAX_USED_BIT: u8 = 6;
    pub const RESERVED_BITMASK: u16 = 0xFFC0;

    #[inline(always)]
    pub const fn get_reserved(num: u16) -> u16 {
        (num & RESERVED_BITMASK) >> MAX_USED_BIT
    }
    #[inline(always)]
    pub const fn set_reserved(num: u16, reserved_in: u16) -> u16 {
        num | (reserved_in << MAX_USED_BIT)
    }
}

/* ------------------------------------------------------------------------- */
/*  HE-AP-info flag word                                                     */
/* ------------------------------------------------------------------------- */

pub mod wifi_he_ap_info {
    /* bits 0-5  →  six-bit BSS-color field (see constant below) */
    pub const partial_bss_color: u8 = 6;
    pub const bss_color_disabled: u8 = 7;

    pub const MAX_USED_BIT: u8 = 8;

    /// `0b0011_1111`
    pub const BSS_COLOR_BITS: u8 = 0x3F;
}

/* ------------------------------------------------------------------------- */
/*  STA-config – **bitfield 1**                                              */
/* ------------------------------------------------------------------------- */

pub mod wifi_sta_config_1 {
    pub const rm_enabled: u8 = 0;
    pub const btm_enabled: u8 = 1;
    pub const mbo_enabled: u8 = 2;
    pub const ft_enabled: u8 = 3;
    pub const owe_enabled: u8 = 4;
    pub const transition_disable: u8 = 5;

    pub const MAX_USED_BIT: u8 = 6;
    pub const RESERVED_BITMASK: u32 = 0xFFFF_FFC0;

    #[inline(always)]
    pub const fn get_reserved(num: u32) -> u32 {
        (num & RESERVED_BITMASK) >> MAX_USED_BIT
    }
    #[inline(always)]
    pub const fn set_reserved(num: u32, reserved_in: u32) -> u32 {
        num | (reserved_in << MAX_USED_BIT)
    }
}

/* ------------------------------------------------------------------------- */
/*  STA-config – **bitfield 2**                                              */
/* ------------------------------------------------------------------------- */
/* Espressif added more bits in IDF v5.5; we reflect that with a Cargo
feature flag `idf_5_5_or_newer`.  */

// pub mod wifi_sta_config_2 {
//     pub const he_dcm_set:                                 u8 = 0;
//     /* multi-bit fields: see constants below */
//     pub const he_dcm_max_constellation_tx_bits:           u8 = 1; /* 2 bits */
//     pub const he_dcm_max_constellation_rx_bits:           u8 = 3; /* 2 bits */
//     pub const he_mcs9_enabled:                            u8 = 5;
//     pub const he_su_beamformee_disabled:                  u8 = 6;
//     pub const he_trig_su_bmforming_feedback_disabled:     u8 = 7;
//     pub const he_trig_mu_bmforming_partial_feedback_disabled: u8 = 8;
//     pub const he_trig_cqi_feedback_disabled:              u8 = 9;
//
//     pub const MAX_USED_BIT:                               u8 = 10;
//     pub const RESERVED_BITMASK:                           u32 = 0xFFFF_FC00;
//
//     #[inline(always)]
//     pub const fn get_reserved(num: u32) -> u32 {
//         (num & RESERVED_BITMASK) >> MAX_USED_BIT
//     }
//     #[inline(always)]
//     pub const fn set_reserved(num: u32, reserved_in: u32) -> u32 {
//         num | (reserved_in << MAX_USED_BIT)
//     }
// }

pub mod wifi_sta_config_2 {
    pub const he_dcm_set: u8 = 0;
    pub const he_dcm_max_constellation_tx_bits: u8 = 1; /* 2 bits */
    pub const he_dcm_max_constellation_rx_bits: u8 = 3; /* 2 bits */
    pub const he_mcs9_enabled: u8 = 5;
    pub const he_su_beamformee_disabled: u8 = 6;
    pub const he_trig_su_bmforming_feedback_disabled: u8 = 7;
    pub const he_trig_mu_bmforming_partial_feedback_disabled: u8 = 8;
    pub const he_trig_cqi_feedback_disabled: u8 = 9;
    pub const vht_su_beamformee_disabled: u8 = 10;
    pub const vht_mu_beamformee_disabled: u8 = 11;
    pub const vht_mcs8_enabled: u8 = 12;

    pub const MAX_USED_BIT: u8 = 13;
    pub const RESERVED_BITMASK: u32 = 0xFFFF_E000;

    #[inline(always)]
    pub const fn get_reserved(num: u32) -> u32 {
        (num & RESERVED_BITMASK) >> MAX_USED_BIT
    }
    #[inline(always)]
    pub const fn set_reserved(num: u32, reserved_in: u32) -> u32 {
        num | (reserved_in << MAX_USED_BIT)
    }
}

/* ------------------------------------------------------------------------- */
/*  Multi-bit-field helper masks                                             */
/* ------------------------------------------------------------------------- */

/* (kept as u8/u32 so callers can OR-in at compile-time) */
pub const WIFI_STA_CONFIG_2_HE_DCM_MAX_CONSTELLATION_TX_MASK: u32 = 0b11 << 1;
pub const WIFI_STA_CONFIG_2_HE_DCM_MAX_CONSTELLATION_RX_MASK: u32 = 0b11 << 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive)]
#[repr(u16)] // todo: Not sure on this one.
enum RpcError {
    RPC_ERR_BASE = ESP_ERR_HOSTED_BASE,
    RPC_ERR_NOT_CONNECTED,
    RPC_ERR_NO_AP_FOUND,
    RPC_ERR_INVALID_PASSWORD,
    RPC_ERR_INVALID_ARGUMENT,
    RPC_ERR_OUT_OF_RANGE,
    RPC_ERR_MEMORY_FAILURE,
    RPC_ERR_UNSUPPORTED_MSG,
    RPC_ERR_INCORRECT_ARG,
    RPC_ERR_PROTOBUF_ENCODE,
    RPC_ERR_PROTOBUF_DECODE,
    RPC_ERR_SET_ASYNC_CB,
    RPC_ERR_TRANSPORT_SEND,
    RPC_ERR_REQUEST_TIMEOUT,
    RPC_ERR_REQ_IN_PROG,
    RPC_ERR_SET_SYNC_SEM,
}
