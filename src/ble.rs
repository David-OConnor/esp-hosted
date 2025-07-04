//! Minimal HCI support for Bluetooth operations

use defmt::{Format, Formatter, println};
use heapless::Vec;
use num_enum::TryFromPrimitive;
use num_traits::float::FloatCore;

use crate::{EspError, ble::HciEvent::CommandComplete};

// todo: Experiment; set these A/R.
const MAX_HCI_EVS: usize = 8;
const MAX_NUM_ADV_DATA: usize = 8;
const MAX_NUM_ADV_REPS: usize = 4;

// For Event Packets (0x04), Byte 0 is the Event Code (e.g. 0x3E for LE Meta‐Event).
// For Command Packets (0x01), Bytes 0–1 together form the OpCode, and Byte 2 is the parameter length.
const HCI_HDR_SIZE: usize = 3;

const HCI_TX_MAX_LEN: usize = 30; // todo: Raise A/R.

#[derive(Clone, Copy, PartialEq, TryFromPrimitive, Format)]
#[repr(u8)]
pub enum HciPkt {
    Cmd = 0x01,
    Acl = 0x02,
    Sco = 0x03,
    Evt = 0x04,
}

#[derive(Clone, Copy, PartialEq, Format, TryFromPrimitive)]
#[repr(u16)]
pub enum HciOpCode {
    LE_SET_SCAN_PARAMS = 0x200B, // OGF 0x08, OCF 0x000B
    LE_SET_SCAN_ENABLE = 0x200C, // OGF 0x08, OCF 0x000B
}

#[derive(Format)]
pub enum AdvData<'a> {
    Flags(u8),
    Complete16BitUuids(&'a [u8]), // len = 2 × n
    Complete32BitUuids(&'a [u8]),
    Complete128BitUuids(&'a [u8]),
    ShortenedLocalName(&'a str),
    CompleteLocalName(&'a str),
    Manufacturer { company: u16, data: &'a [u8] },
    Other { typ: u8, data: &'a [u8] },
}

/// An advertising report
// todo: Derive Format once we get defmt working with Heapless.
// #[derive(Format)]
pub struct AdvReport<'a> {
    pub evt_type: u8,   // ADV_IND, ADV_NONCONN_IND, SCAN_RSP …
    pub addr_type: u8,  // 0 = public, 1 = random, …
    pub addr: [u8; 6],  // LSB first (as on the wire)
    pub data: &'a [u8], // advertising data (slice into original buf)
    pub rssi: i8,       // signed dBm
    pub data_parsed: Vec<AdvData<'a>, MAX_NUM_ADV_DATA>,
}

// todo temp for heapless::Vec missing defmt
impl<'a> Format for AdvReport<'a> {
    fn format(&self, f: Formatter) {
        // Print the header line with fixed fields.
        defmt::write!(
            f,
            "AdvReport {{ evt_type: {}, addr_type: {}, addr: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}, \
             rssi: {} dBm, data_len: {}",
            self.evt_type,
            self.addr_type,
            // reverse for human-friendly big-endian display
            self.addr[5],
            self.addr[4],
            self.addr[3],
            self.addr[2],
            self.addr[1],
            self.addr[0],
            self.rssi,
            self.data.len(),
        );

        // Start the parsed-data list.
        defmt::write!(f, ", data_parsed: \n[");

        // Iterate over every AdvData entry, separated by commas.
        let mut first = true;
        for ad in &self.data_parsed {
            if !first {
                defmt::write!(f, ", ");
            }
            first = false;
            defmt::write!(f, "{}", ad); // assumes AdvData already impls `Format`
        }

        // Close the list and the struct.
        defmt::write!(f, "] }}");
    }
}

#[derive(Clone, Copy, Format, TryFromPrimitive, Default)]
#[repr(u8)]
pub enum BleScanType {
    Passive = 0,
    #[default]
    Active = 1,
}

#[derive(Clone, Copy, Format)]
#[repr(u8)]
pub enum BleOwnAddrType {
    Public = 0,
    Private = 1,
}

#[derive(Clone, Copy, Format)]
#[repr(u8)]
pub enum FilterPolicy {
    AcceptAll = 0,
    WhitelistOnly = 1,
}

pub struct BleScanParams {
    pub scan_type: BleScanType,
    pub interval: u16, // ms
    /// Must be shorter than, or equal to the interval.
    pub window: u16, // ms
    pub own_address_type: BleOwnAddrType,
    pub filter_policy: FilterPolicy,
}

impl BleScanParams {
    pub fn to_bytes(&self) -> [u8; 7] {
        let mut result = [0; 7];

        // Convert to time units of 0.625ms.
        let interval = ((self.interval as f32) / 0.625).round() as u16;
        let window = ((self.window as f32) / 0.625).round() as u16;

        result[0] = self.scan_type as u8;
        result[1..3].copy_from_slice(&interval.to_le_bytes());
        result[3..5].copy_from_slice(&window.to_le_bytes());
        result[5] = self.own_address_type as u8;
        result[6] = self.filter_policy as u8;

        result
    }
}

/// Build helper to push (pkt_type, opcode, params) into an ESP-Hosted frame
pub fn make_hci_cmd(opcode: HciOpCode, params: &[u8]) -> ([u8; HCI_TX_MAX_LEN], usize) {
    let mut payload = [0; HCI_TX_MAX_LEN];

    // payload[0] = HciPkt::Cmd as u8;
    payload[0..2].copy_from_slice(&(opcode as u16).to_le_bytes());
    payload[2] = params.len() as u8;
    payload[3..3 + params.len()].copy_from_slice(params);

    // println!("Writing HCI payload: {:?}", payload[..3 + params.len()]);

    (payload, HCI_HDR_SIZE + params.len())
}

pub fn parse_adv_data(mut d: &[u8]) -> Vec<AdvData<'_>, MAX_NUM_ADV_DATA> {
    let mut result = Vec::<AdvData, MAX_NUM_ADV_DATA>::new();

    while !d.is_empty() {
        let len = d[0] as usize;
        if len == 0 || len > d.len() - 1 {
            break;
        }

        let ad_type = d[1];
        let val = &d[2..1 + len];

        match ad_type {
            0x01 if val.len() == 1 => {
                let _ = result.push(AdvData::Flags(val[0]));
            }
            0x03 => {
                let _ = result.push(AdvData::Complete16BitUuids(val));
            }
            0x05 => {
                let _ = result.push(AdvData::Complete32BitUuids(val));
            }
            0x07 => {
                let _ = result.push(AdvData::Complete128BitUuids(val));
            }
            0x08 => {
                if let Ok(s) = core::str::from_utf8(val) {
                    let _ = result.push(AdvData::ShortenedLocalName(s));
                }
            }
            0x09 => {
                if let Ok(s) = core::str::from_utf8(val) {
                    let _ = result.push(AdvData::CompleteLocalName(s));
                }
            }
            0xFF if val.len() >= 2 => {
                let company = u16::from_le_bytes([val[0], val[1]]);
                let _ = result.push(AdvData::Manufacturer {
                    company,
                    data: &val[2..],
                });
            }
            _ => {
                let _ = result.push(AdvData::Other {
                    typ: ad_type,
                    data: val,
                });
            }
        }

        d = &d[1 + len..];
    }

    result
}

// #[derive(Format)]
pub enum HciEvent<'a> {
    CommandComplete {
        n_cmd: u8, // todo: Is this the cmd?
        opcode: HciOpCode,
        status: u8,
        rest: &'a [u8],
    },
    AdvertisingReport {
        reports: Vec<AdvReport<'a>, MAX_NUM_ADV_REPS>, // up to 4 reports per event
    },
    Unknown {
        evt: u8,
        params: &'a [u8],
    },
}

// todo: Until format works on heapless::Vec.
impl<'a> Format for HciEvent<'a> {
    fn format(&self, fmt: Formatter) {
        match self {
            HciEvent::CommandComplete {
                n_cmd,
                opcode,
                status,
                rest,
            } => {
                defmt::write!(
                    fmt,
                    "CommandComplete {{ n_cmd: {}, opcode: {}, status: {}, rest: {=[u8]} }}",
                    *n_cmd,
                    *opcode,
                    *status,
                    rest
                );
            }
            HciEvent::AdvertisingReport { reports } => {
                // Vec<AdvReport> doesn’t impl Format, so just show how many reports we have
                defmt::write!(fmt, "Advertising reports:");
                for rep in reports {
                    defmt::write!(fmt, "\n-{}; ", rep);
                }
            }
            HciEvent::Unknown { evt, params } => {
                defmt::write!(fmt, "Unknown {{ evt: {}, params: {=[u8]} }}", *evt, params);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Format, TryFromPrimitive)]
#[repr(u8)]
pub enum HciEventType {
    InquiryComplete = 0x01,
    InquiryResult = 0x02,
    ConnectionComplete = 0x03,
    ConnectionRequest = 0x04,
    CommandComplete = 0x0E,
    LeAdvertising = 0x3E,
    // todo: more A/R
}

pub fn parse_hci_events(buf: &[u8]) -> Result<Vec<HciEvent, MAX_HCI_EVS>, EspError> {
    let mut result = Vec::<HciEvent, 8>::new();

    let mut i = 0;

    while i + HCI_HDR_SIZE <= buf.len() {
        // todo: if this crashes, do <
        // Parse all packets present in this payload.
        if buf[i] != HciPkt::Evt as u8 {
            // println!("Non-event HCI packet: {:?}", buf[i..i + 10]);
            return Ok(result);
        }

        // Parse the HCI header.
        let evt_type: HciEventType = match buf[i + 1].try_into() {
            Ok(evt) => evt,
            Err(e) => {
                println!("Error parsing HCI event: {:?}", buf[i + 1]); // todo temp
                return Err(EspError::InvalidData);
            }
        };

        let packet_len = buf[i + 2] as usize;

        if i + 3 + packet_len > buf.len() {
            println!("Buf not long enough for HCI event");
            return Err(EspError::InvalidData);
        }

        let params = &buf[i + 3..i + 3 + packet_len];

        match evt_type {
            HciEventType::CommandComplete => {
                let n_cmd = params[0];
                let opcode: HciOpCode = u16::from_le_bytes([params[1], params[2]])
                    .try_into()
                    .map_err(|_| EspError::InvalidData)?;

                let status = params[3];
                result
                    .push(HciEvent::CommandComplete {
                        n_cmd,
                        opcode,
                        status,
                        rest: &params[4..],
                    })
                    .ok();
            }

            //  LE Advertising Report
            HciEventType::LeAdvertising => {
                if params[0] == 0x02 {
                    // sub-event 0x02, params[1] = number of reports
                    let num = params[1] as usize;
                    let mut idx = 2;
                    let mut reports = Vec::<AdvReport, 4>::new();

                    for _ in 0..num {
                        // minimum bytes per report: 1(evt) + 1(addr_t) + 6(addr)
                        // + 1(data_len) + 0(data) + 1(rssi) = 10
                        if idx + 10 > params.len() {
                            break;
                        }

                        let evt_type = params[idx];
                        idx += 1;
                        let addr_type = params[idx];
                        idx += 1;

                        let mut addr = [0u8; 6];
                        addr.copy_from_slice(&params[idx..idx + 6]);
                        idx += 6;

                        let data_len = params[idx] as usize;
                        idx += 1;
                        if idx + data_len + 1 > params.len() {
                            break;
                        }

                        let data = &params[idx..idx + data_len];
                        idx += data_len;

                        let rssi = params[idx] as i8;
                        idx += 1;

                        reports
                            .push(AdvReport {
                                evt_type,
                                addr_type,
                                addr,
                                data,
                                rssi,
                                data_parsed: parse_adv_data(data),
                            })
                            .ok();
                    }

                    result.push(HciEvent::AdvertisingReport { reports }).ok();
                }
            }

            _ => {
                println!("\n\nUnknown HCI evt type: {:?}", evt_type);

                if result
                    .push(HciEvent::Unknown {
                        evt: evt_type as u8,
                        params,
                    })
                    .is_err()
                {
                    return Err(EspError::Capacity);
                }
            }
        }

        i += HCI_HDR_SIZE + packet_len;
    }

    println!("Num HCI packets in buf: {:?}", result.len()); // todo temp

    Ok(result)
}
