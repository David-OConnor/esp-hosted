/// The configuration here primarily deals with static allocations: By not using an allocator,
/// we must specify the capacity for each repeated, and bytes field. (string as well, but ESP-hosted
/// doesn't use those at this time; uses bytes).

use micropb_gen::{Config, EncodeDecode, Generator};

// SSID, PW and BSSID limits defined in comments in the .proto file.
const SSID_LEN: u32 = 33;
const BSSID_LEN: u32 = 6;
const PW_LEN: u32 = 64;

const PROMISCUOUS_PKT_LEN: u32 = 500; // todo: A/R.
const OTA_DATA_LEN: u32 = 500; // todo: A/R.

// todo: A/R.
const CSI_BUF_LEN: u32 = 30;


fn main() {
    let mut gen_ = Generator::new();
    gen_.use_container_heapless();

    // Default for items not specified below. Raise individual field capacities
    // as required.
    gen_.configure(".", Config::new().max_bytes(16).max_len(8));

    gen_.configure(".wifi_country.cc", Config::new().max_bytes(3));
    gen_.configure(".Rpc_Req_WifiSetCountryCode.country", Config::new().max_bytes(3));
    gen_.configure(".Rpc_Resp_WifiGetCountryCode.country", Config::new().max_bytes(3));

    gen_.configure(".Rpc_Resp_WifiScanGetApRecords.ap_records", Config::new().max_len(30));
    gen_.configure(".wifi_sta_list.sta", Config::new().max_len(20));
    //
    gen_.configure(".wifi_scan_config.bssid", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".wifi_scan_config.ssid", Config::new().max_bytes(SSID_LEN));
    //
    gen_.configure(".wifi_ap_record.bssid", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".wifi_ap_record.ssid", Config::new().max_bytes(SSID_LEN));
    //
    gen_.configure(".wifi_ap_config.password", Config::new().max_bytes(PW_LEN));
    gen_.configure(".wifi_ap_config.ssid", Config::new().max_bytes(SSID_LEN));
    //
    gen_.configure(".wifi_sta_config.bssid", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".wifi_sta_config.password", Config::new().max_bytes(PW_LEN));
    gen_.configure(".wifi_sta_config.ssid", Config::new().max_bytes(SSID_LEN));
    //
    gen_.configure(".wifi_sta_info.mac", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".wifi_ap_config.ssid", Config::new().max_bytes(SSID_LEN));
    //
    gen_.configure(".wifi_event_sta_connected.bssid", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".wifi_event_sta_connected.ssid", Config::new().max_bytes(SSID_LEN));
    gen_.configure(".wifi_event_sta_disconnected.bssid", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".wifi_event_sta_disconnected.ssid", Config::new().max_bytes(SSID_LEN));
    //
    gen_.configure(".wifi_promiscuous_pkt.payload", Config::new().max_bytes(PROMISCUOUS_PKT_LEN));
    gen_.configure(".wifi_csi_info.mac", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".wifi_csi_info.dmac", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".wifi_csi_info.buf", Config::new().max_bytes(CSI_BUF_LEN));
    //
    gen_.configure(".wifi_action_tx_req.dest_mac", Config::new().max_bytes(BSSID_LEN));
    //
    gen_.configure(".wifi_ftm_initiator_cfg.resp_mac", Config::new().max_bytes(BSSID_LEN));
    // Assigned in proto comment
    gen_.configure(".wifi_event_sta_wps_er_pin.pin_code", Config::new().max_bytes(8));
    //
    gen_.configure(".ap_cred.ssid", Config::new().max_bytes(SSID_LEN));
    gen_.configure(".ap_cred.passphrase", Config::new().max_bytes(PW_LEN));
    //
    gen_.configure(".wifi_event_ftm_report.peer_mac", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".wifi_event_action_tx_status.da", Config::new().max_bytes(BSSID_LEN));
    // Assigned in proto comment
    gen_.configure(".wifi_event_ap_wps_rg_pin.pin_code", Config::new().max_bytes(8));
    gen_.configure(".wifi_event_ap_wps_rg_fail_reason.peer_macaddr", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".wifi_event_ap_wps_rg_success.peer_macaddr", Config::new().max_bytes(BSSID_LEN));
    //
    gen_.configure(".Rpc_Resp_GetMacAddress.mac", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".Rpc_Req_SetMacAddress.mac", Config::new().max_bytes(BSSID_LEN));
    //
    gen_.configure(".Rpc_Req_OTAWrite.ota_data", Config::new().max_bytes(OTA_DATA_LEN));
    //
    gen_.configure(".Rpc_Req_WifiApGetStaAid.mac", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".Rpc_Event_AP_StaDisconnected.mac", Config::new().max_bytes(BSSID_LEN));
    gen_.configure(".Rpc_Event_AP_StaConnected.mac", Config::new().max_bytes(BSSID_LEN));


    gen_.compile_protos(&["esp_hosted_rpc.proto"], "../src/proto.rs")
        .unwrap();
}
