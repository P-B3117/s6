use defmt::info;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use esp_radio::ble::controller::BleConnector;
use shared::ble::{GattServer, SENSOR_ADDR};
use trouble_host::prelude::*;

use crate::sensors::SensorDataUpdate;

static CHANNEL: Signal<CriticalSectionRawMutex, SensorDataUpdate> = Signal::new();

pub fn send_message(message: SensorDataUpdate) {
    CHANNEL.signal(message)
}

#[embassy_executor::task]
pub async fn ble_runner(resources: crate::resources::BluetoothResources<'static>) {
    let address: Address = Address::random(SENSOR_ADDR);
    info!("Our address = {:?}", address);

    let connector = BleConnector::new(resources.bt, Default::default()).unwrap();
    let controller: ExternalController<_, 2> = ExternalController::new(connector);

    let mut resources = HostResources::new();
    let stack = trouble_host::new::<_, DefaultPacketPool, 3, 3, 3>(controller, &mut resources)
        .set_random_address(address);
    let mut host = stack.build();

    info!("Starting advertising and GATT service");
    let server = GattServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "Sensor Data",
        appearance: &appearance::sensor::GENERIC_SENSOR,
    }))
    .unwrap();

    let _ = join(host.runner.run(), async {
        loop {
            if let Ok(conn) = advertise(&mut host.peripheral, &server).await {
                loop {
                    if let Err(e) = match CHANNEL.wait().await {
                        SensorDataUpdate::Temperature { temperature } => {
                            server
                                .meteo_service
                                .temperature
                                .notify(&conn, &temperature)
                                .await
                        }
                        SensorDataUpdate::Humidity { humidity } => {
                            server.meteo_service.humidity.notify(&conn, &humidity).await
                        }
                        SensorDataUpdate::Pressure { pressure } => {
                            server.meteo_service.pressure.notify(&conn, &pressure).await
                        }
                        SensorDataUpdate::Light { level } => {
                            server.meteo_service.light_level.notify(&conn, &level).await
                        }
                        SensorDataUpdate::WindDirection { direction } => {
                            server
                                .meteo_service
                                .wind_direction
                                .notify(&conn, &direction)
                                .await
                        }
                        SensorDataUpdate::WindSpeed { speed } => {
                            server.meteo_service.wind_speed.notify(&conn, &speed).await
                        }
                        SensorDataUpdate::Precipitation { mm } => {
                            server.meteo_service.precipitation.notify(&conn, &mm).await
                        }
                    } {
                        esp_println::println!("BLE notify error: {:?}", e);
                    }

                    Timer::after_secs(2).await;
                }
            }
        }
    })
    .await;
}

async fn advertise<'values, 'server, C: Controller>(
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server GattServer<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    let service_uuid = service::ENVIRONMENTAL_SENSING.to_u16().to_le_bytes();
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[service_uuid]),
            AdStructure::CompleteLocalName(b"APP 1 Sensor"),
        ],
        &mut advertiser_data[..],
    )?;
    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..len],
                scan_data: &[],
            },
        )
        .await?;
    info!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}
