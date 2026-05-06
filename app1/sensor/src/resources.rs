use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pipe::Pipe};
use esp_hal::assign_resources;

use shared::*;

assign_resources! {
    pub Resources<'d> {
        bt: BluetoothResources<'d> {
            bt:  BT,
        },
    }
}

// UART

// Declare Pipe sync primitive to share data among Tx and Rx tasks
pub static DATAPIPE0: Pipe<CriticalSectionRawMutex, PIPE_SIZE> = Pipe::new();

static MAP0: phf::Map<
    &'static [u8; BUF_SIZE],
    &'static fn([u8; BUF_SIZE]) -> Result<[u8; BUF_SIZE], &'static str>,
> = phf::phf_map! {
    b"hello00000000000" => b"hello00000000000",
    b"lol0000000000000" => b"wow0000000000000",
};

pub fn executor0(input: [u8; BUF_SIZE]) -> Result<[u8; BUF_SIZE], &'static str> {
    if let Some(output) = MAP0.get(&input) {
        Ok(output(input)?)
    } else {
        esp_println::println!("Base station doesn't know how to handle: {:?}", input);
        Err("Negative number!")
    }
}
