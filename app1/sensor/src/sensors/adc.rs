use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};

use crate::sensors::SensorDataUpdate;

const DIRECTION_PULLUP: f32 = 10_000.0;
const DIRECTION_MAX_ADC: f32 = 4095.0;

#[embassy_executor::task]
pub async fn runner(
    resources: crate::resources::AdcResources<'static>,
    update_data: &'static Channel<CriticalSectionRawMutex, SensorDataUpdate, 10>,
) {
    let mut adc_config = AdcConfig::new();
    let mut light_pin = adc_config.enable_pin(resources.light, Attenuation::_11dB);
    let mut direction_pin = adc_config.enable_pin(resources.direction, Attenuation::_11dB);
    let mut adc = Adc::new(resources.adc, adc_config);

    loop {
        Timer::after_secs(1).await;

        if let Ok(value) = adc.read_oneshot(&mut light_pin) {
            let level = value as f32 / 26.; // Max is ~2600
            update_data
                .send(SensorDataUpdate::Light { level: level as u8 })
                .await;
        }

        if let Ok(value) = adc.read_oneshot(&mut direction_pin) {
            let resistance = DIRECTION_PULLUP * value as f32 / (DIRECTION_MAX_ADC - value as f32);
            let direction = resistance_to_direction(resistance);

            update_data
                .send(SensorDataUpdate::WindDirection { direction })
                .await;
        }
    }
}

const DIRECTIONS: &[(f32, f32)] = &[
    (0.0, 33000.0),
    (22.5, 6570.0),
    (45.0, 8200.0),
    (67.5, 891.0),
    (90.0, 1000.0),
    (112.5, 688.0),
    (135.0, 2200.0),
    (157.5, 1410.0),
    (180.0, 3900.0),
    (202.5, 3140.0),
    (225.0, 16000.0),
    (247.5, 14120.0),
    (270.0, 120000.0),
    (292.5, 42120.0),
    (315.0, 64900.0),
    (337.5, 21880.0),
];

/// Find closest direction from resistance.
fn resistance_to_direction(resistance: f32) -> f32 {
    let mut best_angle = 0.0;
    let mut best_error = f32::MAX;

    for &(angle, expected_resistance) in DIRECTIONS {
        let error = (resistance - expected_resistance).abs();

        if error < best_error {
            best_error = error;
            best_angle = angle;
        }
    }

    best_angle
}
