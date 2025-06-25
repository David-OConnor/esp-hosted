//! Minimal HCI support for Bluetooth operations

use defmt::{Format, Formatter, println};
use heapless::Vec;
use num_enum::TryFromPrimitive;

use crate::EspError;

// todo: Experiment; set these A/R.
const MAX_HCI_EVS: usize = 8;
const MAX_NUM_ADV_DATA: usize = 8;
const MAX_NUM_ADV_REPS: usize = 4;

const HCI_TX_MAX_LEN: usize = 15; // todo: Raise A/R.

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

/// Build helper to push (pkt_type, opcode, params) into an ESP-Hosted frame
pub fn make_hci_cmd(opcode: HciOpCode, params: &[u8]) -> ([u8; HCI_TX_MAX_LEN], usize) {
    let mut payload = [0; HCI_TX_MAX_LEN];

    // payload[0] = HciPkt::Cmd as u8;
    payload[0..2].copy_from_slice(&(opcode as u16).to_le_bytes());
    payload[2] = params.len() as u8;
    payload[3..3 + params.len()].copy_from_slice(params);

    println!("Writing HCI payload: {:?}", payload[..3 + params.len()]);

    (payload, 3 + params.len())
}

pub fn parse_adv_data(mut d: &[u8]) -> Vec<AdvData<'_>, MAX_NUM_ADV_DATA> {
    use heapless::Vec;
    let mut out = Vec::<AdvData, MAX_NUM_ADV_DATA>::new();

    while !d.is_empty() {
        let len = d[0] as usize;
        if len == 0 || len > d.len() - 1 {
            break;
        }

        let ad_type = d[1];
        let val = &d[2..1 + len];

        match ad_type {
            0x01 if val.len() == 1 => {
                let _ = out.push(AdvData::Flags(val[0]));
            }
            0x03 => {
                let _ = out.push(AdvData::Complete16BitUuids(val));
            }
            0x05 => {
                let _ = out.push(AdvData::Complete32BitUuids(val));
            }
            0x07 => {
                let _ = out.push(AdvData::Complete128BitUuids(val));
            }
            0x08 => {
                if let Ok(s) = core::str::from_utf8(val) {
                    let _ = out.push(AdvData::ShortenedLocalName(s));
                }
            }
            0x09 => {
                if let Ok(s) = core::str::from_utf8(val) {
                    let _ = out.push(AdvData::CompleteLocalName(s));
                }
            }
            0xFF if val.len() >= 2 => {
                let company = u16::from_le_bytes([val[0], val[1]]);
                let _ = out.push(AdvData::Manufacturer {
                    company,
                    data: &val[2..],
                });
            }
            _ => {
                let _ = out.push(AdvData::Other {
                    typ: ad_type,
                    data: val,
                });
            }
        }

        d = &d[1 + len..];
    }

    out
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

pub fn parse_hci_events(mut buf: &[u8]) -> Result<Vec<HciEvent, MAX_HCI_EVS>, EspError> {
    let mut out = Vec::<HciEvent, 8>::new();

    while buf.len() >= 3 {
        // Each HCI event in ESP-Hosted must start with 0x04 (H:4 Event)
        if buf[0] != 0x04 {
            break;
        }

        let evt = buf[1];
        let plen = buf[2] as usize;
        if buf.len() < 3 + plen {
            break;
        }

        let params = &buf[3..3 + plen];

        match evt {
            // ────────────────────────── Command Complete ───────────────────
            0x0E if plen >= 4 => {
                let n_cmd = params[0];
                let opcode: HciOpCode = u16::from_le_bytes([params[1], params[2]])
                    .try_into()
                    .map_err(|_| EspError::InvalidData)?;

                let status = params[3];
                out.push(HciEvent::CommandComplete {
                    n_cmd,
                    opcode,
                    status,
                    rest: &params[4..],
                })
                .ok();
            }

            // ─────────────────────── LE Advertising Report ──────────────────
            0x3E if plen >= 2 && params[0] == 0x02 => {
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

                out.push(HciEvent::AdvertisingReport { reports }).ok();
            }

            // ───────────────────────────── default / unknown ────────────────
            _ => {
                println!("\n\nUnknown packet: {:?}\n\n", buf);
                out.push(HciEvent::Unknown { evt, params }).ok();
            }
        }

        // advance to the next event in the buffer
        buf = &buf[3 + plen..];
    }

    Ok(out)
}
