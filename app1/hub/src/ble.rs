use defmt::info;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_radio::ble::controller::BleConnector;
use shared::ble::adv::parse_message_from_adv;
use shared::data::MeteoData;
use trouble_host::prelude::*;

const SENSOR_ADDR: [u8; 6] = [0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff];
static CHANNEL: Channel<CriticalSectionRawMutex, MeteoData, 2> = Channel::new();

pub async fn next_message() -> MeteoData {
    CHANNEL.receive().await
}

#[embassy_executor::task]
pub async fn ble_runner(resources: crate::resources::BluetoothResources<'static>) {
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe6, 0xff]);
    info!("Our address = {:?}", address.to_bytes());

    let connector = BleConnector::new(resources.bt, Default::default()).unwrap();
    let controller: ExternalController<_, 2> = ExternalController::new(connector);

    let mut resources = HostResources::new();
    let stack = trouble_host::new::<_, DefaultPacketPool, 0, 0, 27>(controller, &mut resources)
        .set_random_address(address);
    let mut host = stack.build();

    info!("Starting scanner");
    let printer = Printer;
    let mut scanner = Scanner::new(host.central);
    let _ = join(host.runner.run_with_handler(&printer), async {
        let config = ScanConfig::default();
        let session = Scanner::scan(&mut scanner, &config).await.unwrap();
        info!("scan session started");
        session
    })
    .await;
}

struct Printer;

impl EventHandler for Printer {
    fn on_adv_reports(&self, mut it: LeAdvReportsIter<'_>) {
        while let Some(Ok(report)) = it.next() {
            if report.addr.raw() != SENSOR_ADDR {
                continue;
            }

            if let Some(message) = parse_message_from_adv(report.data) {
                let _ = CHANNEL.try_send(message);
            }
        }
    }
}
