use wincode::{SchemaRead, SchemaWrite};

#[derive(Default, Clone, SchemaWrite, SchemaRead)]
pub struct MeteoData {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub sunlight: f32,
    pub precipitation: f32,
    pub wind_direction: f32,
    pub wind_speed: f32,
}
