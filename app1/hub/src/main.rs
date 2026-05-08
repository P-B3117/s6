#![no_std]
#![no_main]
extern crate alloc;

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::ram;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use shared::data::MeteoData;
use shared::uart::{UartMessage, init_uart, uart_runner_wrapper0, uart_runner_wrapper1};

mod ble;
mod resources;

use resources::*;

use crate::ble::BleDataUpdate;

esp_bootloader_esp_idf::esp_app_desc!();

static SENSOR_RX_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();
static SENSOR_TX_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();
static SERVER_RX_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();
static SERVER_TX_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();

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
    let data = Mutex::<CriticalSectionRawMutex, _>::new(MeteoData::default());

    let uart = init_uart(
        peripherals.UART0,
        peripherals.GPIO1, // TX0D
        peripherals.GPIO3, // RX0D
    );
    spawner.spawn(
        uart_runner_wrapper0(
            uart,
            SERVER_RX_CHANNEL.sender(),
            SERVER_TX_CHANNEL.receiver(),
        )
        .unwrap(),
    );
    info!("Sensor uart initialized!");

    let uart = init_uart(
        peripherals.UART1,
        peripherals.GPIO10, // TX1D
        peripherals.GPIO9,  // RX1D
    );
    spawner.spawn(
        uart_runner_wrapper1(
            uart,
            SENSOR_RX_CHANNEL.sender(),
            SENSOR_TX_CHANNEL.receiver(),
        )
        .unwrap(),
    );
    info!("Server uart initialized!");

    spawner.spawn(ble::ble_runner(resources.bt).unwrap());
    info!("Bluetooth initialized!");

    join3(
        async {
            loop {
                let update = ble::next_message().await;
                let mut data = data.lock().await;
                match update {
                    BleDataUpdate::Temperature { temperature } => data.temperature = temperature,
                    BleDataUpdate::Humidity { humidity } => data.humidity = humidity,
                    BleDataUpdate::Pressure { pressure } => data.pressure = pressure,
                    BleDataUpdate::Light { level } => data.light_level = level,
                    BleDataUpdate::WindDirection { direction } => data.wind_direction = direction,
                    BleDataUpdate::WindSpeed { speed } => data.wind_speed = speed,
                    BleDataUpdate::Precipitation { mm } => data.precipitation = mm,
                }
                SERVER_TX_CHANNEL
                    .send(UartMessage::Data(data.clone()))
                    .await;
            }
        },
        async {
            loop {
                if let UartMessage::AskData = SERVER_RX_CHANNEL.receive().await {
                    let data = data.lock().await.clone();
                    SERVER_TX_CHANNEL.send(UartMessage::Data(data)).await;
                }
            }
        },
        async {
            loop {
                if let UartMessage::Data(new_data) = SENSOR_RX_CHANNEL.receive().await {
                    *data.lock().await = new_data;
                }
            }
        },
    )
    .await;
}
