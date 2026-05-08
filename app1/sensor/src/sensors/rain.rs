use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Instant;
use esp_hal::gpio::{Input, InputConfig};

use crate::sensors::SensorDataUpdate;

#[embassy_executor::task]
pub async fn runner(
    resources: crate::resources::RainResources<'static>,
    update_data: &'static Channel<CriticalSectionRawMutex, SensorDataUpdate, 10>,
) {
    let mut pin = Input::new(resources.pin, InputConfig::default());

    loop {
        pin.wait_for_low().await;
        let start = Instant::now();
        pin.wait_for_rising_edge().await;
        let mm = 0.2794 / start.elapsed().as_millis() as f32;
        update_data
            .send(SensorDataUpdate::Precipitation { mm })
            .await;
    }
}
