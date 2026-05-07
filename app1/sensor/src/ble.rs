use defmt::info;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use esp_radio::ble::controller::BleConnector;
use shared::ble::adv::make_adv;
use shared::data::MeteoData;
use trouble_host::prelude::*;

static CHANNEL: Channel<CriticalSectionRawMutex, MeteoData, 2> = Channel::new();

pub async fn send_message(message: MeteoData) {
    CHANNEL.send(message).await
}

#[embassy_executor::task]
pub async fn ble_runner(
    resources: crate::resources::BluetoothResources<'static>,
    initial_data: MeteoData,
) {
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address.to_bytes());

    let connector = BleConnector::new(resources.bt, Default::default()).unwrap();
    let controller: ExternalController<_, 2> = ExternalController::new(connector);

    let mut resources = HostResources::new();
    let stack = trouble_host::new::<_, DefaultPacketPool, 0, 0, 27>(controller, &mut resources)
        .set_random_address(address);
    let mut host = stack.build();

    let mut adv_data = [0; 64];

    let _ = join(host.runner.run(), async {
        let params = AdvertisementParameters {
            interval_min: Duration::from_millis(50),
            interval_max: Duration::from_millis(250),
            ..Default::default()
        };
        let _advertiser = host
            .peripheral
            .advertise(&params, make_adv(initial_data, &mut adv_data))
            .await
            .unwrap();

        info!("Starting advertising");
        loop {
            let message = CHANNEL.receive().await;
            info!("Updated BLE advertisement data");

            host.peripheral
                .update_adv_data(make_adv(message, &mut adv_data))
                .await
                .unwrap();
        }
    })
    .await;
}
