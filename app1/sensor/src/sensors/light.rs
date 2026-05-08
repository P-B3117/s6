use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};

use crate::sensors::SensorDataUpdate;

#[embassy_executor::task]
pub async fn runner(
    resources: crate::resources::LightResources<'static>,
    update_data: &'static Signal<CriticalSectionRawMutex, SensorDataUpdate>,
) {
    let mut adc_config = AdcConfig::new();
    let mut adc_pin = adc_config.enable_pin(resources.pin, Attenuation::_11dB);
    let mut adc = Adc::new(resources.adc, adc_config);

    loop {
        Timer::after_secs(1).await;

        if let Ok(value) = adc.read_oneshot(&mut adc_pin) {
            let level = value as f32 / 2.6; // Max is ~2600
            update_data.signal(SensorDataUpdate::Light { level: level as u8 });
        }
    }
}
