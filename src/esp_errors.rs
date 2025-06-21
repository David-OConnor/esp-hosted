//! https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/error-codes.html
//!
//! This is incomplete, but has the basics.

#![allow(non_camel_case_types)]

use defmt::Format;
use num_enum::TryFromPrimitive;

// todo: Put back. Flash limit problem. :(

// /// ESP-IDF error codes mapped to a `u16`.
// #[repr(u16)]
// #[derive(Copy, Clone, Format, PartialEq, TryFromPrimitive)]
// pub enum EspCode {
//     /* ── Generic ───────────────────────────────────────────────────────────── */
//     /// Generic failure (`-1` as `0xFFFF` in two-complement form).
//     ESP_FAIL                      = 0xFFFF,
//     /// Success (no error).
//     ESP_OK                        = 0x0000,
//     ESP_ERR_NO_MEM                = 0x0101,
//     ESP_ERR_INVALID_ARG           = 0x0102,
//     ESP_ERR_INVALID_STATE         = 0x0103,
//     ESP_ERR_INVALID_SIZE          = 0x0104,
//     ESP_ERR_NOT_FOUND             = 0x0105,
//     ESP_ERR_NOT_SUPPORTED         = 0x0106,
//     ESP_ERR_TIMEOUT               = 0x0107,
//     ESP_ERR_INVALID_RESPONSE      = 0x0108,
//     ESP_ERR_INVALID_CRC           = 0x0109,
//     ESP_ERR_INVALID_VERSION       = 0x010A,
//     ESP_ERR_INVALID_MAC           = 0x010B,
//     ESP_ERR_NOT_FINISHED          = 0x010C,
//     ESP_ERR_NOT_ALLOWED           = 0x010D,
//
//     /* ── NVS ───────────────────────────────────────────────────────────────── */
//     ESP_ERR_NVS_BASE                  = 0x1100,
//     ESP_ERR_NVS_NOT_INITIALIZED       = 0x1101,
//     ESP_ERR_NVS_NOT_FOUND             = 0x1102,
//     ESP_ERR_NVS_TYPE_MISMATCH         = 0x1103,
//     ESP_ERR_NVS_READ_ONLY             = 0x1104,
//     ESP_ERR_NVS_NOT_ENOUGH_SPACE      = 0x1105,
//     ESP_ERR_NVS_INVALID_NAME          = 0x1106,
//     ESP_ERR_NVS_INVALID_HANDLE        = 0x1107,
//     ESP_ERR_NVS_REMOVE_FAILED         = 0x1108,
//     ESP_ERR_NVS_KEY_TOO_LONG          = 0x1109,
//     ESP_ERR_NVS_PAGE_FULL             = 0x110A,
//     ESP_ERR_NVS_INVALID_STATE         = 0x110B,
//     ESP_ERR_NVS_INVALID_LENGTH        = 0x110C,
//     ESP_ERR_NVS_NO_FREE_PAGES         = 0x110D,
//     ESP_ERR_NVS_VALUE_TOO_LONG        = 0x110E,
//     ESP_ERR_NVS_PART_NOT_FOUND        = 0x110F,
//     ESP_ERR_NVS_NEW_VERSION_FOUND     = 0x1110,
//     ESP_ERR_NVS_XTS_ENCR_FAILED       = 0x1111,
//     ESP_ERR_NVS_XTS_DECR_FAILED       = 0x1112,
//     ESP_ERR_NVS_XTS_CFG_FAILED        = 0x1113,
//     ESP_ERR_NVS_XTS_CFG_NOT_FOUND     = 0x1114,
//     ESP_ERR_NVS_ENCR_NOT_SUPPORTED    = 0x1115,
//     ESP_ERR_NVS_KEYS_NOT_INITIALIZED  = 0x1116,
//     ESP_ERR_NVS_CORRUPT_KEY_PART      = 0x1117,
//     ESP_ERR_NVS_CONTENT_DIFFERS       = 0x1118,
//     ESP_ERR_NVS_WRONG_ENCRYPTION      = 0x1119,
//
//     /* ── ULP ───────────────────────────────────────────────────────────────── */
//     ESP_ERR_ULP_BASE              = 0x1200,
//     ESP_ERR_ULP_SIZE_TOO_BIG      = 0x1201,
//     ESP_ERR_ULP_INVALID_LOAD_ADDR = 0x1202,
//     ESP_ERR_ULP_DUPLICATE_LABEL   = 0x1203,
//     ESP_ERR_ULP_UNDEFINED_LABEL   = 0x1204,
//     ESP_ERR_ULP_BRANCH_OUT_OF_RANGE = 0x1205,
//
//     /* ── OTA ───────────────────────────────────────────────────────────────── */
//     ESP_ERR_OTA_BASE                   = 0x1500,
//     ESP_ERR_OTA_PARTITION_CONFLICT     = 0x1501,
//     ESP_ERR_OTA_SELECT_INFO_INVALID    = 0x1502,
//     ESP_ERR_OTA_VALIDATE_FAILED        = 0x1503,
//     ESP_ERR_OTA_SMALL_SEC_VER          = 0x1504,
//     ESP_ERR_OTA_ROLLBACK_FAILED        = 0x1505,
//     ESP_ERR_OTA_ROLLBACK_INVALID_STATE = 0x1506,
//
//     /* ── eFuse ─────────────────────────────────────────────────────────────── */
//     ESP_ERR_EFUSE                       = 0x1600,
//     ESP_OK_EFUSE_CNT                    = 0x1601,
//     ESP_ERR_EFUSE_CNT_IS_FULL           = 0x1602,
//     ESP_ERR_EFUSE_REPEATED_PROG         = 0x1603,
//     ESP_ERR_CODING                      = 0x1604,
//     ESP_ERR_NOT_ENOUGH_UNUSED_KEY_BLOCKS= 0x1605,
//     ESP_ERR_DAMAGED_READING             = 0x1606,
//
//     /* ── Image loader ──────────────────────────────────────────────────────── */
//     ESP_ERR_IMAGE_BASE         = 0x2000,
//     ESP_ERR_IMAGE_FLASH_FAIL   = 0x2001,
//     ESP_ERR_IMAGE_INVALID      = 0x2002,
//
//     /* ── Wi-Fi ─────────────────────────────────────────────────────────────── */
//     ESP_ERR_WIFI_BASE              = 0x3000,
//     ESP_ERR_WIFI_NOT_INIT          = 0x3001,
//     ESP_ERR_WIFI_NOT_STARTED       = 0x3002,
//     ESP_ERR_WIFI_NOT_STOPPED       = 0x3003,
//     ESP_ERR_WIFI_IF                = 0x3004,
//     ESP_ERR_WIFI_MODE              = 0x3005,
//     ESP_ERR_WIFI_STATE             = 0x3006,
//     ESP_ERR_WIFI_CONN              = 0x3007,
//     ESP_ERR_WIFI_NVS               = 0x3008,
//     ESP_ERR_WIFI_MAC               = 0x3009,
//     ESP_ERR_WIFI_SSID              = 0x300A,
//     ESP_ERR_WIFI_PASSWORD          = 0x300B,
//     ESP_ERR_WIFI_TIMEOUT           = 0x300C,
//     ESP_ERR_WIFI_WAKE_FAIL         = 0x300D,
//     ESP_ERR_WIFI_WOULD_BLOCK       = 0x300E,
//     ESP_ERR_WIFI_NOT_CONNECT       = 0x300F,
//     ESP_ERR_WIFI_POST              = 0x3012,
//     ESP_ERR_WIFI_INIT_STATE        = 0x3013,
//     ESP_ERR_WIFI_STOP_STATE        = 0x3014,
//     ESP_ERR_WIFI_NOT_ASSOC         = 0x3015,
//     ESP_ERR_WIFI_TX_DISALLOW       = 0x3016,
//     ESP_ERR_WIFI_TWT_FULL          = 0x3017,
//     ESP_ERR_WIFI_TWT_SETUP_TIMEOUT = 0x3018,
//     ESP_ERR_WIFI_TWT_SETUP_TXFAIL  = 0x3019,
//     ESP_ERR_WIFI_TWT_SETUP_REJECT  = 0x301A,
//     ESP_ERR_WIFI_DISCARD           = 0x301B,
//     ESP_ERR_WIFI_ROC_IN_PROGRESS   = 0x301C,
//     ESP_ERR_WIFI_REGISTRAR         = 0x3033,
//     ESP_ERR_WIFI_WPS_TYPE          = 0x3034,
//     ESP_ERR_WIFI_WPS_SM            = 0x3035,
//
//     /* ── ESP-NOW ───────────────────────────────────────────────────────────── */
//     ESP_ERR_ESPNOW_BASE      = 0x3064,
//     ESP_ERR_ESPNOW_NOT_INIT  = 0x3065,
//     ESP_ERR_ESPNOW_ARG       = 0x3066,
//     ESP_ERR_ESPNOW_NO_MEM    = 0x3067,
//     ESP_ERR_ESPNOW_FULL      = 0x3068,
//     ESP_ERR_ESPNOW_NOT_FOUND = 0x3069,
//     ESP_ERR_ESPNOW_INTERNAL  = 0x306A,
//     ESP_ERR_ESPNOW_EXIST     = 0x306B,
//     ESP_ERR_ESPNOW_IF        = 0x306C,
// }
