use trouble_host::prelude::*;

pub const SENSOR_ADDR: [u8; 6] = [0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff];

#[gatt_server]
pub struct GattServer {
    pub meteo_service: GattObjectTransferService,
}

#[gatt_service(uuid = service::ENVIRONMENTAL_SENSING)]
pub struct GattObjectTransferService {
    #[characteristic(uuid = characteristic::LUMINOUS_INTENSITY, write, read, notify)]
    pub light_level: u8,
    #[characteristic(uuid = characteristic::TEMPERATURE, write, read, notify)]
    pub temperature: i8,
    #[characteristic(uuid = characteristic::HUMIDITY, write, read, notify)]
    pub humidity: u8,
    #[characteristic(uuid = characteristic::PRESSURE, write, read, notify)]
    pub pressure: u32,
    #[characteristic(uuid = characteristic::RAINFALL, write, read, notify)]
    pub precipitation: f32,
    #[characteristic(uuid = characteristic::APPARENT_WIND_DIRECTION, write, read, notify)]
    pub wind_direction: f32,
    #[characteristic(uuid = characteristic::APPARENT_WIND_SPEED, write, read, notify)]
    pub wind_speed: f32,
    #[characteristic(uuid = characteristic::NEW_ALERT, write, read, notify)]
    pub updates: u32,
}
