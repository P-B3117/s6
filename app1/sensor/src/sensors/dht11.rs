// Inspired from https://github.com/rekkun/esp32-dht11-rs-embassy

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Timer, block_for};
use esp_hal::gpio::{DriveMode, Flex, InputConfig, OutputConfig};

use crate::sensors::SensorDataUpdate;

#[embassy_executor::task]
pub async fn runner(
    resources: crate::resources::DHT11Resources<'static>,
    update_data: &'static Channel<CriticalSectionRawMutex, SensorDataUpdate, 10>,
) {
    let mut buffer = [0u8; 5];
    let mut pin = Flex::new(resources.pin);
    let out_config = OutputConfig::default().with_drive_mode(DriveMode::OpenDrain);
    pin.apply_output_config(&out_config);
    let input_config = InputConfig::default();
    pin.apply_input_config(&input_config);

    'scan: loop {
        Timer::after_secs(2).await;

        pin.set_output_enable(true);
        pin.set_low();
        block_for(Duration::from_millis(20));
        pin.set_high();
        block_for(Duration::from_micros(40));
        pin.set_input_enable(true);

        let start = Instant::now();
        while pin.is_high() {
            if start.elapsed().as_millis() > 100 {
                esp_println::println!("DHT11 Wait for low timeout.");
                continue 'scan;
            }
        }

        if pin.is_low() {
            block_for(Duration::from_micros(80));
            if pin.is_low() {
                esp_println::println!("DHT11 Wait for high timeout.");
                continue 'scan;
            }
        }
        block_for(Duration::from_micros(80));
        buffer.fill(0);
        for byte in 0..5 {
            for bit in 0..8u8 {
                let start = Instant::now();
                while pin.is_low() {
                    if start.elapsed().as_micros() > 100 {
                        esp_println::println!("DHT11 Wait for byte start high timeout.");
                        continue 'scan;
                    }
                }
                block_for(Duration::from_micros(30));
                if pin.is_high() {
                    buffer[byte] |= 1 << (7 - bit);
                } else {
                    let start = Instant::now();
                    while pin.is_high() {
                        if start.elapsed().as_micros() > 1000 {
                            esp_println::println!("DHT11 Wait for byte end low timeout.");
                            continue 'scan;
                        }
                    }
                }
            }
        }

        // Transmision is over
        let humidity = buffer[0];
        // let humidity_decimals = buffer[1];
        let temperature = if buffer[2] & 0x80 == 0 {
            buffer[2] as i8
        } else {
            -(buffer[2] as i8 & 0x7F)
        };
        // let temperature_decimals = buffer[3];
        let checksum = buffer[4];

        let sum = buffer[0]
            .wrapping_add(buffer[1])
            .wrapping_add(buffer[2])
            .wrapping_add(buffer[3]);
        if checksum != sum {
            esp_println::println!("DHT11 checksum failed, {:?}", buffer);
            continue 'scan;
        }

        update_data
            .send(SensorDataUpdate::Temperature { temperature })
            .await;
        update_data
            .send(SensorDataUpdate::Humidity { humidity })
            .await;
    }
}
