use defmt::info;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_radio::ble::controller::BleConnector;
use trouble_host::prelude::*;

type Message = u32;

const COMPANY_ID: u16 = 0xFFF0;
const SENSOR_ADDR: [u8; 6] = [0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff];
static CHANNEL: Channel<CriticalSectionRawMutex, Message, 2> = Channel::new();

pub async fn next_message() -> Message {
    CHANNEL.receive().await
}

#[embassy_executor::task]
pub async fn run(resources: crate::resources::BluetoothResources<'static>) {
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
        let session = scanner.scan(&config).await.unwrap();
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
                info!("adv message from {:?}: {}", report.addr, message);
                let _ = CHANNEL.try_send(message);
            }
        }
    }
}

fn parse_message_from_adv(data: &[u8]) -> Option<Message> {
    parse_ad_structures(data, |ad_type, payload| {
        if ad_type == 0xFF && payload.len() >= 10 {
            let company_id = u16::from_le_bytes([payload[0], payload[1]]);
            if company_id == COMPANY_ID {
                let first = u32::from_be_bytes([payload[2], payload[3], payload[4], payload[5]]);
                let second = u32::from_be_bytes([payload[6], payload[7], payload[8], payload[9]]);
                if first == second {
                    return Some(first);
                }
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
