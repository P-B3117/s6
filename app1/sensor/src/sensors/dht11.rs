use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::{DriveMode, Flex, InputConfig, OutputConfig};

use crate::resources::DHT11Resources;
use crate::sensors::SensorDataUpdate;

#[embassy_executor::task]
pub async fn runner(
    resources: DHT11Resources<'static>,
    update_data: &'static Signal<CriticalSectionRawMutex, SensorDataUpdate>,
) {
    let mut buffer = [0u8; 5];
    let mut pin = Flex::new(resources.pin);
    let out_config = OutputConfig::default().with_drive_mode(DriveMode::OpenDrain);
    pin.apply_output_config(&out_config);
    let input_config = InputConfig::default();
    pin.apply_input_config(&input_config);

    loop {
        Timer::after_secs(1).await;

        // Send start signal
        pin.set_output_enable(true);
        pin.set_low();
        Timer::after_millis(20).await;
        pin.set_high();
        Timer::after_micros(40).await;

        // Wait for response
        pin.set_input_enable(true);
        // 80us low
        pin.wait_for_rising_edge().await;
        // 80us high
        pin.wait_for_falling_edge().await;

        // Read data
        for i in 0..40 {
            Timer::after_micros(50).await;
            buffer[i / 8] <<= 1;

            let start = Instant::now();
            pin.wait_for_falling_edge().await;
            if start.elapsed() > Duration::from_micros(50) {
                // 0b is 26-28us
                // 1b is 70us
                buffer[i / 8] |= 1;
            }
        }

        // Transmision is over
        let humidity = buffer[0];
        // let humidity_decimals = buffer[1];
        let temperature = buffer[2];
        // let temperature_decimals = buffer[3];
        let checksum = buffer[4];

        let sum = buffer[0..=3].iter().sum::<u8>();
        if checksum != (sum & 0xFF) {
            continue;
        }

        update_data.signal(SensorDataUpdate::DHT11 {
            temperature,
            humidity,
        });
    }
}
