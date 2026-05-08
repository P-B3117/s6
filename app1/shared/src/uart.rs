use defmt::info;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Receiver, Sender};
use esp_hal::Async;
use esp_hal::gpio::AnyPin;
use esp_hal::uart::{AtCmdConfig, Config, Instance, RxConfig, Uart};
use serde::{Deserialize, Serialize};

use crate::data::MeteoData;

// Read Buffer Size
pub const BUF_SIZE: usize = 512;
pub const PIPE_SIZE: usize = 128;
pub const UART_SIZE: usize = 128;

// End of Transmission Character (Carrige Return -> 13 or 0x0D in ASCII)
const AT_CMD: u8 = 0x0D;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum UartMessage {
    AskData,
    Data(MeteoData),
}

pub fn init_uart(
    uart: impl Instance + 'static,
    tx_pin: impl Into<AnyPin<'static>>,
    rx_pin: impl Into<AnyPin<'static>>,
) -> Uart<'static, Async> {
    let config = Config::default().with_rx(RxConfig::default());

    info!("UART  config initialized!");

    let mut uart = Uart::new(uart, config)
        .unwrap()
        .with_tx(tx_pin.into())
        .with_rx(rx_pin.into())
        .into_async();

    info!("UART created!");

    uart.set_at_cmd(AtCmdConfig::default().with_cmd_char(AT_CMD));

    info!("UART AT_CMD set!");

    uart
}

#[embassy_executor::task]
pub async fn uart_runner_wrapper0(
    uart: Uart<'static, Async>,
    message_channel: Sender<'static, CriticalSectionRawMutex, UartMessage, 3>,
    send_message_channel: Receiver<'static, CriticalSectionRawMutex, UartMessage, 3>,
) {
    uart_runner(uart, message_channel, send_message_channel).await;
}

#[embassy_executor::task]
pub async fn uart_runner_wrapper1(
    uart: Uart<'static, Async>,
    message_channel: Sender<'static, CriticalSectionRawMutex, UartMessage, 3>,
    send_message_channel: Receiver<'static, CriticalSectionRawMutex, UartMessage, 3>,
) {
    uart_runner(uart, message_channel, send_message_channel).await;
}

pub async fn uart_runner(
    uart: Uart<'static, Async>,
    message_channel: Sender<'static, CriticalSectionRawMutex, UartMessage, 3>,
    send_message_channel: Receiver<'static, CriticalSectionRawMutex, UartMessage, 3>,
) {
    info!("UART start task!");
    let mut read_buffer = [0u8; BUF_SIZE];
    let (mut rx, mut tx) = uart.split();
    info!("UART start loop!");

    join(
        async {
            loop {
                info!("UART reader!");
                match rx.read_async(&mut read_buffer).await {
                    Ok(len) => {
                        let data = serde_json::from_slice(&read_buffer[..len]).unwrap();
                        message_channel.send(data).await;
                    }
                    Err(e) => esp_println::println!("RX Error: {:?}", e),
                }
            }
        },
        async {
            loop {
                info!("UART writer!");
                let data = send_message_channel.receive().await;
                let payload = serde_json::to_string(&data).unwrap();
                tx.write_async(payload.as_bytes()).await.unwrap();
                tx.flush_async().await.unwrap();
            }
        },
    )
    .await;
}
