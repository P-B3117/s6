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

// Declare Pipe sync primitive to share data among Tx and Rx tasks
pub static DATAPIPE0: Pipe<CriticalSectionRawMutex, PIPE_SIZE> = Pipe::new();

pub fn executor0(input: [u8; BUF_SIZE]) -> [u8; BUF_SIZE] {
    let output: [u8; BUF_SIZE];

    match &input {
        b"hello00000000000" => {
            output = *b"hello00000000000";
        }
        _ => {
            output = *b"hello00000000000";
        }
    }

    output
}
