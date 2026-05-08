// Inspired from https://github.com/perrylson/rust-dps310-driver

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;

use crate::sensors::SensorDataUpdate;

const SENSOR_ADDR: u8 = 0x77;

// Registers
const COEFFS_ADDR: u8 = 0x10;

const PRS_CFG_ADDR: u8 = 0x06;
const TMP_CFG_ADDR: u8 = 0x07;
const MEAS_CFG_ADDR: u8 = 0x08;

const PRESSURE_ADDR: u8 = 0x00;
const TEMP_ADDR: u8 = 0x03;

// Oversampling scale factors from datasheet
const SCALE_FACTORS: f64 = 3670016.0;

#[derive(Debug)]
struct Calibration {
    c0: i16,
    c1: i16,
    c00: i32,
    c10: i32,
    c01: i16,
    c11: i16,
    c20: i16,
    c21: i16,
    c30: i16,
}

#[embassy_executor::task]
pub async fn runner(
    resources: crate::resources::DPS310Resources<'static>,
    update_data: &'static Channel<CriticalSectionRawMutex, SensorDataUpdate, 10>,
) {
    let i2c_config = Config::default().with_frequency(Rate::from_khz(400));

    let mut i2c = I2c::new(resources.i2c, i2c_config)
        .unwrap()
        .with_sda(resources.sda)
        .with_scl(resources.scl)
        .into_async();

    // Allow sensor startup
    Timer::after(Duration::from_millis(50)).await;

    // Read calibration coefficients
    let mut coeffs = [0u8; 18];
    i2c.write_read_async(SENSOR_ADDR, &[COEFFS_ADDR], &mut coeffs)
        .await
        .unwrap();
    let cal = parse_calibration(&coeffs);
    esp_println::println!("Calibration: {:?}", cal);

    // Configure sensor (pressure + temperature oversampling)
    // Oversampling x4
    i2c.write_async(SENSOR_ADDR, &[TMP_CFG_ADDR, 0x82])
        .await
        .unwrap();
    i2c.write_async(SENSOR_ADDR, &[PRS_CFG_ADDR, 0x02])
        .await
        .unwrap();

    // Continuous pressure + temperature measurement
    i2c.write_async(SENSOR_ADDR, &[MEAS_CFG_ADDR, 0b0111])
        .await
        .unwrap();

    loop {
        Timer::after(Duration::from_secs(1)).await;

        // Read raw temperature (3 bytes)
        let mut tbuf = [0u8; 3];
        i2c.write_read_async(SENSOR_ADDR, &[TEMP_ADDR], &mut tbuf)
            .await
            .unwrap();
        let raw_temp = sign_extend_24(u32::from_be_bytes([0, tbuf[0], tbuf[1], tbuf[2]]) >> 8);

        // Read raw pressure (3 bytes)
        let mut pbuf = [0u8; 3];
        i2c.write_read_async(SENSOR_ADDR, &[PRESSURE_ADDR], &mut pbuf)
            .await
            .unwrap();
        let raw_press = sign_extend_24(u32::from_be_bytes([0, pbuf[0], pbuf[1], pbuf[2]]) >> 8);

        // Compensation
        let temp_sc = raw_temp as f64 / SCALE_FACTORS;
        let press_sc = raw_press as f64 / SCALE_FACTORS;

        let _temp_comp = cal.c0 as f64 * 0.5 + cal.c1 as f64 * temp_sc;

        let pressure_comp = cal.c00 as f64
            + press_sc * (cal.c10 as f64 + press_sc * (cal.c20 as f64 + press_sc * cal.c30 as f64))
            + temp_sc * (cal.c01 as f64 + press_sc * (cal.c11 as f64 + press_sc * cal.c21 as f64));

        update_data
            .send(SensorDataUpdate::Pressure {
                pressure: (pressure_comp / 10.) as u32,
            })
            .await;
    }
}

// Calibration parsing (datasheet bit packing)
fn parse_calibration(b: &[u8; 18]) -> Calibration {
    Calibration {
        c0: (((b[0] as u16) << 4) | ((b[1] >> 4) as u16)) as i16,
        c1: ((((b[1] & 0x0F) as u16) << 8) | b[2] as u16) as i16,

        c00: sign_extend_20(((b[3] as u32) << 12) | ((b[4] as u32) << 4) | ((b[5] >> 4) as u32)),
        c10: sign_extend_20((((b[5] & 0x0F) as u32) << 16) | ((b[6] as u32) << 8) | b[7] as u32),

        c01: ((b[8] as u16) << 8 | b[9] as u16) as i16,
        c11: ((b[10] as u16) << 8 | b[11] as u16) as i16,
        c20: ((b[12] as u16) << 8 | b[13] as u16) as i16,
        c21: ((b[14] as u16) << 8 | b[15] as u16) as i16,
        c30: ((b[16] as u16) << 8 | b[17] as u16) as i16,
    }
}

// 24-bit sign extension
fn sign_extend_24(v: u32) -> i32 {
    ((v << 8) as i32) >> 8
}

// 20-bit sign extension (used for c00/c10)
fn sign_extend_20(v: u32) -> i32 {
    ((v << 12) as i32) >> 12
}
