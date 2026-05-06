use defmt::info;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use esp_radio::ble::controller::BleConnector;
use trouble_host::prelude::*;

type Message = u32;

const COMPANY_ID: u16 = 0xFFF0;
static CHANNEL: Channel<CriticalSectionRawMutex, Message, 2> = Channel::new();

pub async fn send_message(message: Message) {
    CHANNEL.send(message).await
}

#[embassy_executor::task]
pub async fn run(resources: crate::resources::BluetoothResources<'static>) {
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address.to_bytes());

    let connector = BleConnector::new(resources.bt, Default::default()).unwrap();
    let controller: ExternalController<_, 2> = ExternalController::new(connector);

    let mut resources = HostResources::new();
    let stack = trouble_host::new::<_, DefaultPacketPool, 0, 0, 27>(controller, &mut resources)
        .set_random_address(address);
    let mut host = stack.build();

    let mut adv_data = [0; 64];
    let mut update_count = 0u32;

    let _ = join(host.runner.run(), async {
        let mut params = AdvertisementParameters::default();
        params.interval_min = Duration::from_millis(50);
        params.interval_max = Duration::from_millis(250);
        let _advertiser = host
            .peripheral
            .advertise(&params, make_advertisement(update_count, &mut adv_data))
            .await
            .unwrap();

        info!("Starting advertising");
        loop {
            let message = CHANNEL.receive().await;
            update_count = update_count.wrapping_add(1);

            host.peripheral
                .update_adv_data(make_advertisement(message, &mut adv_data))
                .await
                .unwrap();

            info!("Still running: Updated the beacon {} times", update_count);
        }
    })
    .await;
}

fn make_advertisement<'d>(message: Message, buffer: &'d mut [u8]) -> Advertisement<'d> {
    let mut payload = [0u8; 8];
    payload[0..4].copy_from_slice(&message.to_be_bytes());
    payload[4..8].copy_from_slice(&message.to_be_bytes());

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
