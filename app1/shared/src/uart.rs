use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Receiver, Sender};
use esp_hal::Async;
use esp_hal::gpio::AnyPin;
use esp_hal::uart::{AtCmdConfig, Config, Instance, RxConfig, Uart};
use wincode::io::Cursor;
use wincode::{SchemaRead, SchemaWrite};

use crate::data::MeteoData;

// Read Buffer Size
pub const BUF_SIZE: usize = 16;
pub const PIPE_SIZE: usize = 128;
pub const UART_SIZE: usize = 128;
const AT_CMD: u8 = '\r' as u8;

#[derive(SchemaWrite, SchemaRead)]
pub enum UartMessage {
    AskData,
    Data(MeteoData),
}

pub fn init_uart(
    uart: impl Instance + 'static,
    tx_pin: impl Into<AnyPin<'static>>,
    rx_pin: impl Into<AnyPin<'static>>,
) -> Uart<'static, Async> {
    let config =
        Config::default().with_rx(RxConfig::default().with_fifo_full_threshold(UART_SIZE as u16));

    let mut uart = Uart::new(uart, config)
        .unwrap()
        .with_tx(tx_pin.into())
        .with_rx(rx_pin.into())
        .into_async();
    uart.set_at_cmd(AtCmdConfig::default().with_cmd_char(AT_CMD));
    uart
}

#[embassy_executor::task]
pub async fn uart_runner(
    uart: Uart<'static, Async>,
    message_channel: Sender<'static, CriticalSectionRawMutex, UartMessage, 3>,
    send_message_channel: Receiver<'static, CriticalSectionRawMutex, UartMessage, 3>,
) {
    let mut read_buffer = [0u8; BUF_SIZE];
    let mut write_buffer = [0u8; BUF_SIZE];
    let (mut rx, mut tx) = uart.split();

    join(
        async {
            loop {
                match rx.read_async(&mut read_buffer).await {
                    Ok(len) => {
                        let data = wincode::deserialize(&read_buffer[..len]).unwrap();
                        message_channel.send(data).await;
                    }
                    Err(e) => esp_println::println!("RX Error: {:?}", e),
                }
            }
        },
        async {
            loop {
                let data = send_message_channel.receive().await;
                wincode::serialize_into(&mut Cursor::new(&mut write_buffer[..]), &data).unwrap();
                tx.write_async(&write_buffer).await.unwrap();
                tx.flush_async().await.unwrap();
            }
        },
    )
    .await;
}
