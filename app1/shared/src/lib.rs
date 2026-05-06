#![no_std]

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pipe::Pipe};
use embedded_io_async::Read;
use esp_backtrace as _;
use esp_backtrace as _;
use esp_hal::Async;
use esp_hal::gpio::AnyPin;
use esp_hal::uart::{AtCmdConfig, Config, Instance, RxConfig, Uart, UartRx, UartTx};
use heapless::String;

// Read Buffer Size
pub const BUF_SIZE: usize = 16;
pub const PIPE_SIZE: usize = 128;
pub const UART_SIZE: usize = 128;

// End of Transmission Character (Carrige Return -> 13 or 0x0D in ASCII)
const AT_CMD: u8 = 0x0D;

pub async fn get_uart(
    uart: impl Instance + 'static,
    tx_pin: AnyPin<'static>,
    rx_pin: AnyPin<'static>,
) -> Uart<'static, Async> {
    let config =
        Config::default().with_rx(RxConfig::default().with_fifo_full_threshold(UART_SIZE as u16));

    let mut uart0 = Uart::new(uart, config)
        .unwrap()
        .with_tx(tx_pin)
        .with_rx(rx_pin)
        .into_async();
    uart0.set_at_cmd(AtCmdConfig::default().with_cmd_char(AT_CMD));

    uart0
}

//TODO: use the datapipe to actually write the reader's answer
#[embassy_executor::task]
pub async fn writer(
    mut tx: UartTx<'static, Async>,
    mut pipe: &'static Pipe<CriticalSectionRawMutex, PIPE_SIZE>,
) {
    let mut wbuf: [u8; BUF_SIZE] = [0u8; BUF_SIZE];

    embedded_io_async::Write::write(&mut tx, b"Initializing writer\r\n")
        .await
        .unwrap();
    embedded_io_async::Write::flush(&mut tx).await.unwrap();

    loop {
        // copy from pipe into wbuf
        let r = pipe.read_exact(&mut wbuf).await;
        match r {
            Ok(_) => {
                // write wbuf to tx
                embedded_io_async::Write::write_all(&mut tx, &mut wbuf)
                    .await
                    .unwrap();

                // flush tx to send the data
                embedded_io_async::Write::flush(&mut tx).await.unwrap();
            }
            Err(e) => {
                esp_println::println!("Writer Error: {:?}", e);
            }
        }
    }
}

#[embassy_executor::task]
pub async fn reader(
    mut rx: UartRx<'static, Async>,
    pipe: &'static Pipe<CriticalSectionRawMutex, PIPE_SIZE>,
    executor: fn([u8; BUF_SIZE]) -> Result<[u8; BUF_SIZE], &'static str>,
) {
    // Declare read buffer to store Rx characters
    let mut rbuf: [u8; BUF_SIZE] = [0u8; BUF_SIZE];
    loop {
        // Read characters from UART into read buffer until EOT
        let r = embedded_io_async::Read::read(&mut rx, &mut rbuf[0..]).await;
        match r {
            Ok(_) => {
                let processed = executor(rbuf);
                // If read succeeds then write recieved characters to pipe
                match processed {
                    Ok(processed) => pipe.write_all(&processed).await,
                    Err(e) => esp_println::println!("Executor Error: {}", e),
                }
            }
            Err(e) => esp_println::println!("RX Error: {:?}", e),
        }
    }
}
