#![no_std]
#![no_main]
extern crate alloc;

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::ram;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use shared::data::MeteoData;
use shared::uart::{UartMessage, init_uart, uart_runner_wrapper0};

mod ble;
mod resources;
mod sensors;

use resources::*;

use crate::sensors::SensorDataUpdate;

esp_bootloader_esp_idf::esp_app_desc!();

static HUB_SENSOR_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();
static SENSOR_HUB_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();
static UPDATE_DATA: Channel<CriticalSectionRawMutex, SensorDataUpdate, 10> = Channel::new();

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
        peripherals.GPIO12, // TX1D
        peripherals.GPIO39, // RX1D
    );
    spawner.spawn(
        uart_runner_wrapper0(
            uart,
            HUB_SENSOR_CHANNEL.sender(),
            SENSOR_HUB_CHANNEL.receiver(),
        )
        .unwrap(),
    );
    info!("Uart initialized!");

    spawner.spawn(ble::ble_runner(resources.bt).unwrap());
    info!("Bluetooth initialized!");

    spawner.spawn(sensors::dht11::runner(resources.dht11, &UPDATE_DATA).unwrap());
    spawner.spawn(sensors::dps310::runner(resources.dps310, &UPDATE_DATA).unwrap());
    spawner.spawn(sensors::adc::runner(resources.adc, &UPDATE_DATA).unwrap());
    spawner.spawn(sensors::rain::runner(resources.rain, &UPDATE_DATA).unwrap());
    spawner.spawn(sensors::wind::runner(resources.wind, &UPDATE_DATA).unwrap());
    info!("Sensors initialized!");

    let data = Mutex::<NoopRawMutex, _>::new(MeteoData::default());
    join(
        async {
            loop {
                let update = UPDATE_DATA.receive().await;
                esp_println::println!("Received update: {:?}\r", &update);
                let mut data = data.lock().await;
                match update {
                    SensorDataUpdate::Temperature { temperature } => data.temperature = temperature,
                    SensorDataUpdate::Humidity { humidity } => data.humidity = humidity,
                    SensorDataUpdate::Pressure { pressure } => data.pressure = pressure,
                    SensorDataUpdate::Light { level } => data.light_level = level,
                    SensorDataUpdate::WindDirection { direction } => {
                        data.wind_direction = direction
                    }
                    SensorDataUpdate::WindSpeed { speed } => data.wind_speed = speed,
                    SensorDataUpdate::Precipitation { mm } => data.precipitation = mm,
                }
                ble::send_message(update);
            }
        },
        async {
            esp_println::println!("Starting Uart injector loop");
            loop {
                if let UartMessage::AskData = HUB_SENSOR_CHANNEL.receive().await {
                    esp_println::println!("got data from uart");
                    let mut data = data.lock().await.clone();
                    data.u = 1;
                    esp_println::println!("sending to hub\r");
                    SENSOR_HUB_CHANNEL.send(UartMessage::Data(data)).await;
                }
            }
        },
    )
    .await;
}
