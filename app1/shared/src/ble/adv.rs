use crate::ble::COMPANY_ID;
use crate::data::MeteoData;
use trouble_host::prelude::*;
use wincode::io::Cursor;

pub fn make_adv<'d>(message: MeteoData, buffer: &'d mut [u8]) -> Advertisement<'d> {
    let mut payload = [0u8; 8];
    wincode::serialize_into(Cursor::new(&mut payload[..]), &message).unwrap();

    let len = AdStructure::encode_slice(
        &[
            AdStructure::CompleteLocalName(b"Trouble Beacon"),
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ManufacturerSpecificData {
                company_identifier: COMPANY_ID,
                payload: &payload,
            },
        ],
        &mut buffer[..],
    )
    .unwrap();

    Advertisement::NonconnectableNonscannableUndirected {
        adv_data: &buffer[..len],
    }
}

pub fn parse_message_from_adv(data: &[u8]) -> Option<MeteoData> {
    parse_ad_structures(data, |ad_type, payload| {
        if ad_type == 0xFF && payload.len() >= 10 {
            let company_id = u16::from_le_bytes([payload[0], payload[1]]);
            if company_id == COMPANY_ID {
                let data = wincode::deserialize(&payload[2..12]).ok()?;
                return Some(data);
            }
        }
        None
    })
}

fn parse_ad_structures<T, F>(data: &[u8], mut f: F) -> Option<T>
where
    F: FnMut(u8, &[u8]) -> Option<T>,
{
    let mut i = 0;
    while i < data.len() {
        let len = data[i] as usize;
        if len == 0 {
            break;
        }
        let end = i + 1 + len;
        if end > data.len() {
            break;
        }
        let ad_type = data[i + 1];
        let payload = &data[i + 2..end];
        if let Some(value) = f(ad_type, payload) {
            return Some(value);
        }
        i = end;
    }
    None
}
