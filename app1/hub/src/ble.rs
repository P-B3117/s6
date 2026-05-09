use defmt::info;
use embassy_futures::join::join;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use esp_radio::ble::controller::BleConnector;
use shared::ble::SENSOR_ADDR;
use shared::data::MeteoData;
use trouble_host::prelude::*;

static CHANNEL: Channel<CriticalSectionRawMutex, MeteoData, 10> = Channel::new();

pub async fn next_message() -> MeteoData {
    CHANNEL.receive().await
}

fn decode_u32(value: &[u8]) -> Option<u32> {
    let bytes: [u8; 4] = value.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn decode_f32(value: &[u8]) -> Option<f32> {
    let bytes: [u8; 4] = value.try_into().ok()?;
    Some(f32::from_le_bytes(bytes))
}

async fn read_full_snapshot<C, P, const MAX_SERVICES: usize>(
    client: &GattClient<'_, C, P, MAX_SERVICES>,
    light_level: &Characteristic<u8>,
    temperature: &Characteristic<i8>,
    humidity: &Characteristic<u8>,
    pressure: &Characteristic<u32>,
    precipitation: &Characteristic<f32>,
    wind_direction: &Characteristic<f32>,
    wind_speed: &Characteristic<f32>,
) where
    C: Controller,
    P: PacketPool,
{
    let mut buf_1 = [0u8; 1];
    let mut buf_4 = [0u8; 4];
    let mut snapshot = MeteoData::default();

    if let Ok(1) = client.read_characteristic(light_level, &mut buf_1).await {
        snapshot.light_level = buf_1[0];
    }

    if let Ok(1) = client.read_characteristic(temperature, &mut buf_1).await {
        snapshot.temperature = buf_1[0] as i8;
    }

    if let Ok(1) = client.read_characteristic(humidity, &mut buf_1).await {
        snapshot.humidity = buf_1[0];
    }

    if let Ok(4) = client.read_characteristic(pressure, &mut buf_4).await {
        if let Some(value) = decode_u32(&buf_4) {
            snapshot.pressure = value;
        }
    }

    if let Ok(4) = client.read_characteristic(precipitation, &mut buf_4).await {
        if let Some(mm) = decode_f32(&buf_4) {
            snapshot.precipitation = mm;
        }
    }

    if let Ok(4) = client.read_characteristic(wind_direction, &mut buf_4).await {
        if let Some(direction) = decode_f32(&buf_4) {
            snapshot.wind_direction = direction;
        }
    }

    if let Ok(4) = client.read_characteristic(wind_speed, &mut buf_4).await {
        if let Some(speed) = decode_f32(&buf_4) {
            snapshot.wind_speed = speed;
        }
    }

    CHANNEL.send(snapshot).await;
}

#[embassy_executor::task]
pub async fn ble_runner(resources: crate::resources::BluetoothResources<'static>) {
    let address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address);

    let connector = BleConnector::new(resources.bt, Default::default()).unwrap();
    let controller = ExternalController::<_, 20>::new(connector);

    let mut resources = HostResources::new();
    let stack = trouble_host::new::<_, DefaultPacketPool, 8, 8, 8>(controller, &mut resources)
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

    let _ = join(host.runner.run(), async {
        loop {
            info!("Connecting");

            let conn = match host.central.connect(&config).await {
                Ok(conn) => conn,
                Err(err) => {
                    info!("Connect failed: {:?}", err);
                    Timer::after_secs(2).await;
                    continue;
                }
            };
            info!("Connected, creating gatt client");

            let client = match GattClient::<_, DefaultPacketPool, 16>::new(&stack, &conn).await {
                Ok(client) => client,
                Err(err) => {
                    info!("Gatt client init failed: {:?}", err);
                    conn.disconnect();
                    Timer::after_secs(2).await;
                    continue;
                }
            };

            let notifications = async {
                info!("Looking for environmental sensing service");
                let services = match client
                    .services_by_uuid(&Uuid::new_short(service::ENVIRONMENTAL_SENSING.to_u16()))
                    .await
                {
                    Ok(services) => services,
                    Err(err) => {
                        info!("Service discovery failed: {:?}", err);
                        return;
                    }
                };
                let Some(service) = services.first().cloned() else {
                    info!("Environmental sensing service not found");
                    return;
                };

                info!("Discovering characteristics");
                let light_level_characteristic = client
                    .characteristic_by_uuid::<u8>(
                        &service,
                        &Uuid::new_short(characteristic::LUMINOUS_INTENSITY.to_u16()),
                    )
                    .await
                    .unwrap();

                let temperature_characteristic = client
                    .characteristic_by_uuid::<i8>(
                        &service,
                        &Uuid::new_short(characteristic::TEMPERATURE.to_u16()),
                    )
                    .await
                    .unwrap();

                let humidity_characteristic = client
                    .characteristic_by_uuid::<u8>(
                        &service,
                        &Uuid::new_short(characteristic::HUMIDITY.to_u16()),
                    )
                    .await
                    .unwrap();

                let pressure_characteristic = client
                    .characteristic_by_uuid::<u32>(
                        &service,
                        &Uuid::new_short(characteristic::PRESSURE.to_u16()),
                    )
                    .await
                    .unwrap();

                let precipitation_characteristic = client
                    .characteristic_by_uuid::<f32>(
                        &service,
                        &Uuid::new_short(characteristic::RAINFALL.to_u16()),
                    )
                    .await
                    .unwrap();

                let wind_direction_characteristic = client
                    .characteristic_by_uuid::<f32>(
                        &service,
                        &Uuid::new_short(characteristic::APPARENT_WIND_DIRECTION.to_u16()),
                    )
                    .await
                    .unwrap();

                let wind_speed_characteristic = client
                    .characteristic_by_uuid::<f32>(
                        &service,
                        &Uuid::new_short(characteristic::APPARENT_WIND_SPEED.to_u16()),
                    )
                    .await
                    .unwrap();

                info!("Subscribing to notifications");
                let updates_characteristic = client
                    .characteristic_by_uuid::<f32>(
                        &service,
                        &Uuid::new_short(characteristic::NEW_ALERT.to_u16()),
                    )
                    .await
                    .unwrap();
                let mut notifications = client
                    .subscribe(&updates_characteristic, false)
                    .await
                    .unwrap();

                info!("Updates subscription ready");

                loop {
                    match select(conn.next(), notifications.next()).await {
                        Either::First(ConnectionEvent::Disconnected { reason }) => {
                            info!("Disconnected: {:?}", reason);
                            break;
                        }
                        Either::First(_) => {}
                        Either::Second(_) => {
                            read_full_snapshot(
                                &client,
                                &light_level_characteristic,
                                &temperature_characteristic,
                                &humidity_characteristic,
                                &pressure_characteristic,
                                &precipitation_characteristic,
                                &wind_direction_characteristic,
                                &wind_speed_characteristic,
                            )
                            .await;
                        }
                    }
                }
            };

            match select(client.task(), notifications).await {
                Either::First(Err(err)) => info!("Gatt client task failed: {:?}", err),
                _ => {}
            }

            conn.disconnect();
            Timer::after_secs(2).await;
        }
    })
    .await;
}
