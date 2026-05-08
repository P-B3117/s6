pub mod dht11;
pub mod dps310;
pub mod adc;
pub mod rain;
pub mod wind;

#[derive(Debug)]
pub enum SensorDataUpdate {
    DHT11 { temperature: i8, humidity: u8 },
    DPS310 { pressure: u32 },
    Light { level: u8 },
    WindDirection { direction: f32 },
    WindSpeed { speed: f32 },
    Precipitation { mm: f32 },
}
