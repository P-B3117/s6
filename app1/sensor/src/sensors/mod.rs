pub mod adc;
pub mod dht11;
pub mod dps310;
pub mod rain;
pub mod wind;

#[derive(Debug)]
pub enum SensorDataUpdate {
    Temperature { temperature: i8 },
    Humidity { humidity: u8 },
    Pressure { pressure: u32 },
    Light { level: u8 },
    WindDirection { direction: f32 },
    WindSpeed { speed: f32 },
    Precipitation { mm: f32 },
}
