use crate::ble::COMPANY_ID;
use crate::data::MeteoData;
use trouble_host::prelude::*;

pub fn make_adv<'d>(message: MeteoData, buffer: &'d mut [u8]) -> Advertisement<'d> {
    let payload = serde_json::to_string(&message).unwrap();

    let len = AdStructure::encode_slice(
        &[
            AdStructure::CompleteLocalName(b"Trouble Beacon"),
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ManufacturerSpecificData {
                company_identifier: COMPANY_ID,
                payload: payload.as_bytes(),
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
    for ad in AdStructure::decode(data) {
        if let Ok(AdStructure::ManufacturerSpecificData {
            company_identifier,
            payload,
        }) = ad
        {
            if company_identifier == COMPANY_ID {
                let data = serde_json::from_slice(payload).ok()?;
                return Some(data);
            }
        }
    }
    None
}
