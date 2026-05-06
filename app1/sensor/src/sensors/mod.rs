pub mod dht11;

pub enum SensorDataUpdate {
    DHT11 { temperature: u8, humidity: u8 },
}
