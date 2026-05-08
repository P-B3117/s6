use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MeteoData {
    pub light_level: u8,     // percents
    pub temperature: i8,     // celsius
    pub humidity: u8,        // percents
    pub pressure: u32,       // hPa
    pub precipitation: f32,  // mm/s
    pub wind_direction: f32, // degrees
    pub wind_speed: f32,     // km/h
}
