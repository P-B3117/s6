use wincode::{SchemaRead, SchemaWrite};

#[derive(Default, Clone, SchemaWrite, SchemaRead)]
pub struct MeteoData {
    pub light_level: u8,
    pub temperature: i8,
    pub humidity: u8,
    pub pressure: u32,
    pub precipitation: f32,
    pub wind_direction: f32,
    pub wind_speed: f32,
}
