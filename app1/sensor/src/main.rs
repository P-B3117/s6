#![no_std]
#![no_main]
extern crate alloc;

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::ram;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;

mod ble;
mod resources;

use resources::*;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let resources = split_resources!(peripherals);
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64000);
    esp_rtos::start(
        TimerGroup::new(peripherals.TIMG0).timer0,
        SoftwareInterruptControl::new(peripherals.SW_INTERRUPT).software_interrupt0,
    );
    info!("Embassy initialized!");

    spawner.spawn(ble::run(resources.bt).unwrap());

    loop {
        ble::send_message(2).await;
        Timer::after(Duration::from_secs(3)).await;
        ble::send_message(3).await;
        Timer::after(Duration::from_secs(3)).await;
        ble::send_message(4).await;
        Timer::after(Duration::from_secs(3)).await;
    }
}
