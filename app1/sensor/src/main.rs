#![no_std]
#![no_main]
extern crate alloc;

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::ram;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use shared::data::MeteoData;
use shared::uart::{UartMessage, init_uart, uart_runner};

mod ble;
mod resources;
mod sensors;

use resources::*;

use crate::sensors::SensorDataUpdate;

esp_bootloader_esp_idf::esp_app_desc!();

static HUB_RX_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();
static HUB_TX_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();
static UPDATE_DATA: Signal<CriticalSectionRawMutex, SensorDataUpdate> = Signal::new();

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

    let uart = init_uart(
        peripherals.UART1,
        peripherals.GPIO10, // TX1D
        peripherals.GPIO9,  // RX1D
    );
    spawner.spawn(uart_runner(uart, HUB_RX_CHANNEL.sender(), HUB_TX_CHANNEL.receiver()).unwrap());
    info!("Uart initialized!");

    spawner.spawn(ble::ble_runner(resources.bt, MeteoData::default()).unwrap());
    info!("Bluetooth initialized!");

    spawner.spawn(sensors::dht11::runner(resources.dht11, &UPDATE_DATA).unwrap());
    spawner.spawn(sensors::dps310::runner(resources.dps310, &UPDATE_DATA).unwrap());
    spawner.spawn(sensors::light::runner(resources.light, &UPDATE_DATA).unwrap());
    info!("Sensors initialized!");

    let data = Mutex::<NoopRawMutex, _>::new(MeteoData::default());
    join(
        async {
            loop {
                let update = UPDATE_DATA.wait().await;
                esp_println::println!("Received update: {:?}", &update);
                let mut data = data.lock().await;
                match update {
                    SensorDataUpdate::DHT11 {
                        temperature,
                        humidity,
                    } => {
                        data.humidity = humidity;
                        data.temperature = temperature;
                    }
                    SensorDataUpdate::DPS310 { pressure } => {
                        data.pressure = pressure;
                    }
                    SensorDataUpdate::Light { level } => {
                        data.light_level = level;
                    }
                }
                ble::send_message(data.clone()).await;
            }
        },
        async {
            loop {
                match HUB_RX_CHANNEL.receive().await {
                    UartMessage::AskData => {
                        let data = data.lock().await.clone();
                        HUB_TX_CHANNEL.send(UartMessage::Data(data)).await;
                    }
                    _ => {}
                }
            }
        },
    )
    .await;
}
