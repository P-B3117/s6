use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Instant, Timer};
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::gpio::{Input, InputConfig};

use crate::sensors::SensorDataUpdate;

const DIRECTION_PULLUP: f32 = 10_000.0;
const DIRECTION_MAX_ADC: f32 = 4095.0;

#[embassy_executor::task]
pub async fn runner(
    resources: crate::resources::WindResources<'static>,
    update_data: &'static Signal<CriticalSectionRawMutex, SensorDataUpdate>,
) {
    let mut speed_pin = Input::new(resources.speed, InputConfig::default());

    let mut adc_config = AdcConfig::new();
    let mut direction_pin = adc_config.enable_pin(resources.direction, Attenuation::_11dB);
    let mut adc = Adc::new(resources.adc, adc_config);

    join(
        async {
            loop {
                speed_pin.wait_for_low().await;
                let start = Instant::now();
                speed_pin.wait_for_rising_edge().await;
                let speed = 2.4 / start.elapsed().as_millis() as f32;
                update_data.signal(SensorDataUpdate::WindSpeed { speed });
            }
        },
        async {
            loop {
                Timer::after_secs(1).await;

                if let Ok(value) = adc.read_oneshot(&mut direction_pin) {
                    let resistance =
                        DIRECTION_PULLUP * value as f32 / (DIRECTION_MAX_ADC - value as f32);
                    let direction = resistance_to_direction(resistance);

                    update_data.signal(SensorDataUpdate::WindDirection { direction });
                }
            }
        },
    )
    .await;
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
