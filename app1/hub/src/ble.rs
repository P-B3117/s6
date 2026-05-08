use defmt::info;
use embassy_futures::join::join;
use embassy_futures::select::{Either, Either6, select, select6};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_radio::ble::controller::BleConnector;
use shared::ble::SENSOR_ADDR;
use trouble_host::prelude::*;

pub enum BleDataUpdate {
    Temperature { temperature: i8 },
    Humidity { humidity: u8 },
    Pressure { pressure: u32 },
    Light { level: u8 },
    WindDirection { direction: f32 },
    WindSpeed { speed: f32 },
    Precipitation { mm: f32 },
}

static CHANNEL: Channel<CriticalSectionRawMutex, BleDataUpdate, 10> = Channel::new();

pub async fn next_message() -> BleDataUpdate {
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

#[embassy_executor::task]
pub async fn ble_runner(resources: crate::resources::BluetoothResources<'static>) {
    let address = Address::random([0xff, 0x8f, 0x1b, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address);

    let connector = BleConnector::new(resources.bt, Default::default()).unwrap();
    let controller = ExternalController::<_, 20>::new(connector);

    let mut resources = HostResources::new();
    let stack = trouble_host::new::<_, DefaultPacketPool, 3, 3, 3>(controller, &mut resources)
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
            info!("Looking for environmental sensing service");
            let services = client
                .services_by_uuid(&Uuid::new_short(service::ENVIRONMENTAL_SENSING.to_u16()))
                .await
                .unwrap();
            let service = services.first().unwrap().clone();

            info!("Subscribing notifications");
            let characteristic = client
                .characteristic_by_uuid::<u8>(
                    &service,
                    &Uuid::new_short(characteristic::LUMINOUS_INTENSITY.to_u16()),
                )
                .await
                .unwrap();
            let mut light_level = client.subscribe(&characteristic, false).await.unwrap();

            let characteristic = client
                .characteristic_by_uuid::<i8>(
                    &service,
                    &Uuid::new_short(characteristic::TEMPERATURE.to_u16()),
                )
                .await
                .unwrap();
            let mut temperature = client.subscribe(&characteristic, false).await.unwrap();

            let characteristic = client
                .characteristic_by_uuid::<u8>(
                    &service,
                    &Uuid::new_short(characteristic::HUMIDITY.to_u16()),
                )
                .await
                .unwrap();
            let mut humidity = client.subscribe(&characteristic, false).await.unwrap();

            let characteristic = client
                .characteristic_by_uuid::<u32>(
                    &service,
                    &Uuid::new_short(characteristic::PRESSURE.to_u16()),
                )
                .await
                .unwrap();
            let mut pressure = client.subscribe(&characteristic, false).await.unwrap();

            let characteristic = client
                .characteristic_by_uuid::<f32>(
                    &service,
                    &Uuid::new_short(characteristic::RAINFALL.to_u16()),
                )
                .await
                .unwrap();
            let mut precipitation = client.subscribe(&characteristic, false).await.unwrap();

            let characteristic = client
                .characteristic_by_uuid::<f32>(
                    &service,
                    &Uuid::new_short(characteristic::APPARENT_WIND_DIRECTION.to_u16()),
                )
                .await
                .unwrap();
            let mut wind_direction = client.subscribe(&characteristic, false).await.unwrap();

            let characteristic = client
                .characteristic_by_uuid::<f32>(
                    &service,
                    &Uuid::new_short(characteristic::APPARENT_WIND_SPEED.to_u16()),
                )
                .await
                .unwrap();
            let mut wind_speed = client.subscribe(&characteristic, false).await.unwrap();

            info!("All subscribtions done");

            loop {
                match select6(
                    light_level.next(),
                    temperature.next(),
                    humidity.next(),
                    pressure.next(),
                    precipitation.next(),
                    select(wind_direction.next(), wind_speed.next()),
                )
                .await
                {
                    Either6::First(light_level) => {
                        CHANNEL
                            .send(BleDataUpdate::Light {
                                level: light_level.as_ref()[0],
                            })
                            .await
                    }
                    Either6::Second(temperature) => {
                        CHANNEL
                            .send(BleDataUpdate::Temperature {
                                temperature: temperature.as_ref()[0] as i8,
                            })
                            .await
                    }
                    Either6::Third(humidity) => {
                        CHANNEL
                            .send(BleDataUpdate::Humidity {
                                humidity: humidity.as_ref()[0],
                            })
                            .await
                    }
                    Either6::Fourth(pressure) => {
                        if let Some(pressure) = decode_u32(pressure.as_ref()) {
                            CHANNEL.send(BleDataUpdate::Pressure { pressure }).await
                        } else {
                            info!(
                                "Pressure update had invalid length: {}",
                                pressure.as_ref().len()
                            );
                        }
                    }
                    Either6::Fifth(precipitation) => {
                        if let Some(mm) = decode_f32(precipitation.as_ref()) {
                            CHANNEL.send(BleDataUpdate::Precipitation { mm }).await
                        } else {
                            info!(
                                "Precipitation update had invalid length: {}",
                                precipitation.as_ref().len()
                            );
                        }
                    }
                    Either6::Sixth(Either::First(wind_direction)) => {
                        if let Some(direction) = decode_f32(wind_direction.as_ref()) {
                            CHANNEL
                                .send(BleDataUpdate::WindDirection { direction })
                                .await
                        } else {
                            info!(
                                "Wind direction update had invalid length: {}",
                                wind_direction.as_ref().len()
                            );
                        }
                    }
                    Either6::Sixth(Either::Second(wind_speed)) => {
                        if let Some(speed) = decode_f32(wind_speed.as_ref()) {
                            CHANNEL.send(BleDataUpdate::WindSpeed { speed }).await
                        } else {
                            info!(
                                "Wind speed update had invalid length: {}",
                                wind_speed.as_ref().len()
                            );
                        }
                    }
                }
            }
        })
        .await;
    })
    .await;
}
