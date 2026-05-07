pub mod dht11;
pub mod dps310;
pub mod light;

#[derive(Debug)]
pub enum SensorDataUpdate {
    DHT11 { temperature: i8, humidity: u8 },
    DPS310 { pressure: u32 },
    Light { level: u8 },
}
