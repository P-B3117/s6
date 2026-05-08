use defmt::info;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use esp_radio::ble::controller::BleConnector;
use shared::ble::SENSOR_ADDR;
use shared::data::MeteoData;
use trouble_host::prelude::*;

static CHANNEL: Channel<CriticalSectionRawMutex, MeteoData, 2> = Channel::new();

pub async fn next_message() -> MeteoData {
    CHANNEL.receive().await
}

#[embassy_executor::task]
pub async fn ble_runner(resources: crate::resources::BluetoothResources<'static>) {
    let address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address);

    let connector = BleConnector::new(resources.bt, Default::default()).unwrap();
    let controller = ExternalController::<_, 20>::new(connector);

    let mut resources = HostResources::new();
    let stack = trouble_host::new::<_, DefaultPacketPool, 0, 3, 20>(controller, &mut resources)
        .set_random_address(address);
    let mut host = stack.build();

    let target = Address::random(SENSOR_ADDR);
    let config = ConnectConfig {
        connect_params: Default::default(),
        scan_config: ScanConfig {
            filter_accept_list: &[(target.kind, &target.addr)],
            ..Default::default()
        },
    };

    info!("Scanning for peripheral...");
    let _ = join(host.runner.run(), async {
        info!("Connecting");

        let conn = host.central.connect(&config).await.unwrap();
        info!("Connected, creating gatt client");

        let client = GattClient::<_, DefaultPacketPool, 10>::new(&stack, &conn)
            .await
            .unwrap();

        let _ = join(client.task(), async {
            info!("Looking for battery service");
            let services = client
                .services_by_uuid(&Uuid::new_short(0x180f))
                .await
                .unwrap();
            let service = services.first().unwrap().clone();

            info!("Looking for value handle");
            let c: Characteristic<u8> = client
                .characteristic_by_uuid(&service, &Uuid::new_short(0x2a19))
                .await
                .unwrap();

            info!("Subscribing notifications");
            let mut listener = client.subscribe(&c, false).await.unwrap();

            let _ = join(
                async {
                    loop {
                        let mut data = [0; 1];
                        client.read_characteristic(&c, &mut data[..]).await.unwrap();
                        info!("Read value: {}", data[0]);
                        Timer::after_secs(10).await;
                    }
                },
                async {
                    loop {
                        let data = listener.next().await;
                        info!(
                            "Got notification: {:?} (val: {})",
                            data.as_ref(),
                            data.as_ref()[0]
                        );
                    }
                },
            )
            .await;
        })
        .await;
    })
    .await;
}
