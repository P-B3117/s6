#![no_std]
#![no_main]
extern crate alloc;

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Duration;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;

mod resources;
use resources::*;

use shared::*;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let resources = split_resources!(peripherals);
    esp_alloc::heap_allocator!(size: 128 * 1024);
    esp_rtos::start(
        TimerGroup::new(peripherals.TIMG0).timer0,
        SoftwareInterruptControl::new(peripherals.SW_INTERRUPT).software_interrupt0,
    );
    info!("Embassy initialized!");

    //BLE

    //UART

    let uart0 = get_uart(
        peripherals.UART0,
        peripherals.GPIO1.into(), // TX0D
        peripherals.GPIO3.into(), // RX0D
    )
    .await;

    let (rx, tx) = uart0.split();
    spawner.spawn(writer(tx, &DATAPIPE0).unwrap());
    spawner.spawn(reader(rx, &DATAPIPE0, executor0).unwrap());
}
