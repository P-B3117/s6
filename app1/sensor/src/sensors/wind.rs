use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Instant;
use esp_hal::gpio::{Input, InputConfig};

use crate::sensors::SensorDataUpdate;

#[embassy_executor::task]
pub async fn runner(
    resources: crate::resources::WindResources<'static>,
    update_data: &'static Channel<CriticalSectionRawMutex, SensorDataUpdate, 10>,
) {
    let mut speed_pin = Input::new(resources.speed, InputConfig::default());

    loop {
        speed_pin.wait_for_low().await;
        let start = Instant::now();
        speed_pin.wait_for_rising_edge().await;
        let speed = 2400.0 / start.elapsed().as_millis() as f32;
        update_data
            .send(SensorDataUpdate::WindSpeed { speed })
            .await;
    }
}
