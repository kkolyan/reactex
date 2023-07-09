use std::str::from_utf8;
use raw_sync::Timeout::Infinite;
use serde::{Deserialize, Serialize};
use crate::shmem_lab::{ShmemReceiver, ShmemSender};
use crate::utils::{PingMessage, read_u32_le, write_u32_le};

pub const MESSAGE_SHMEM_SIZE: usize = 512;


pub fn shmem_ping_send(m: &PingMessage, sender: &ShmemSender<[u8; MESSAGE_SHMEM_SIZE]>) {
    let mut data = [0u8; MESSAGE_SHMEM_SIZE];
    let json_string = serde_json::to_string(&m).unwrap();
    let json = json_string.as_bytes();
    write_u32_le(&mut &mut data[0..4], json.len() as u32).unwrap();
    data[4..json.len() + 4].copy_from_slice(json);
    sender.send(data, Infinite).unwrap();
}

pub fn shmem_ping_receive(receiver: &ShmemReceiver<[u8; MESSAGE_SHMEM_SIZE]>) -> PingMessage {
    let response = receiver.receive(Infinite).unwrap();
    let n = read_u32_le(&mut &response[0..4]).unwrap();

    let end = n as usize + 4;
    let v = &response[4..(end)];
    let result = from_utf8(v).unwrap();
    serde_json::from_str(result).unwrap()
}