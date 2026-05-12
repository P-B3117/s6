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

esp_bootloader_esp_idf::esp_app_desc!();

static SENSOR_HUB_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();
static HUB_SENSOR_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();
static SERVER_HUB_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();
static HUB_SERVER_CHANNEL: Channel<CriticalSectionRawMutex, UartMessage, 3> = Channel::new();

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
        false,
    );
    spawner.spawn(
        uart_runner_wrapper0(
            uart,
            SERVER_HUB_CHANNEL.sender(), // receive from server and send to channel
            HUB_SERVER_CHANNEL.receiver(), // take the message  from channel and send to server
        )
        .unwrap(),
    );
    info!("Sensor uart initialized!");

    let uart1 = init_uart(
        peripherals.UART1,
        peripherals.GPIO14, // TX1D
        peripherals.GPIO12, // RX1D
        true,
    );
    spawner.spawn(
        uart_runner_wrapper1(
            uart1,
            SENSOR_HUB_CHANNEL.sender(), // receive from sensor and send to channel
            HUB_SENSOR_CHANNEL.receiver(), // take the message from channel and send to sensor
        )
        .unwrap(),
    );
    info!("Server uart initialized!");

    spawner.spawn(ble::ble_runner(resources.bt).unwrap());
    info!("Bluetooth initialized!");

    join3(
        async {
            // BLE to server loop
            loop {
                let snapshot = ble::next_message().await; // when we get a new BLE message
                let mut data = data.lock().await;
                *data = snapshot;
                // send data to server
                HUB_SERVER_CHANNEL
                    .send(UartMessage::Data(data.clone()))
                    .await;
            }
        },
        async {
            // Hub to server loop
            loop {
                // if server asks for data
                if let UartMessage::AskData = SERVER_HUB_CHANNEL.receive().await {
                    let data = data.lock().await.clone();
                    // send last data to server
                    // HUB_SERVER_CHANNEL.send(UartMessage::Data(data)).await;
                    esp_println::println!("Asking for data from sensor\r");
                    // ask sensor for data, will receive a response soon
                    HUB_SENSOR_CHANNEL.send(UartMessage::AskData).await;
                }
            }
        },
        async {
            loop {
                if let UartMessage::Data(new_data) = SENSOR_HUB_CHANNEL.receive().await {
                    esp_println::println!("RECEIVED for data from sensor\r");
                    let to_send = UartMessage::Data(new_data.clone());
                    // update local data
                    *data.lock().await = new_data;
                    // send data to server
                    HUB_SERVER_CHANNEL.send(to_send).await;
                }
            }
        },
    )
    .await;
}
