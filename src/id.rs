use std::{thread, time::Duration};

use wasm_timer::{SystemTime, UNIX_EPOCH};

use crate::{SocketCapability, TLSVersion};

#[derive(Clone)]
pub struct ConnIdFactory {
    last_time: SystemTime,
    incr: u8,
}

impl ConnIdFactory {
    pub fn new() -> Self {
        Self {
            last_time: SystemTime::now(),
            incr: 0,
        }
    }

    pub fn generate(&mut self, conn_type: SocketCapability) -> ConnId {
        let since = SystemTime::now().duration_since(self.last_time).unwrap();
        let conn_type: u8 = conn_type.into();

        if since.as_millis() == 0 {
            if self.incr == u8::MAX {
                thread::sleep(Duration::from_millis(1));
                self.incr = 0;
            } else {
                self.incr = self.incr + 1;
            }
        } else {
            self.incr = 0;
        }

        self.last_time = SystemTime::now();
        ConnId {
            time: self
                .last_time
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            conn_type,
            incr: self.incr,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ConnId {
    /// Time in ms (first 48 bits)
    pub time: u64,
    /// Connection type (8 bits)
    pub conn_type: u8,
    /// Incremental fallback when multiple ids are created in 1ms (8 bits)
    pub incr: u8,
}

impl Into<u64> for ConnId {
    fn into(self) -> u64 {
        ((self.time as u64) << 16) | ((self.conn_type as u64) << 8) | (self.incr as u64)
    }
}

impl From<u64> for ConnId {
    fn from(value: u64) -> Self {
        let time: u64 = (value >> 16) as u64;
        let conn_type: u8 = (((value >> 8) & 0xFF) as u8).into();
        let incr: u8 = (value & 0xFF) as u8;
        Self {
            time,
            conn_type,
            incr,
        }
    }
}

impl From<u8> for SocketCapability {
    fn from(value: u8) -> Self {
        match value {
            0 => SocketCapability::TCP,
            10 => SocketCapability::HTTP,
            20 => SocketCapability::HTTPS(TLSVersion::TLSv1_0),
            21 => SocketCapability::HTTPS(TLSVersion::TLSv1_1),
            22 => SocketCapability::HTTPS(TLSVersion::TLSv1_2),
            23 => SocketCapability::HTTPS(TLSVersion::TLSv1_3),
            _ => panic!("Invalid socket capability"),
        }
    }
}

impl Into<u8> for SocketCapability {
    fn into(self) -> u8 {
        match self {
            SocketCapability::TCP => 0,
            SocketCapability::HTTP => 10,
            SocketCapability::HTTPS(TLSVersion::TLSv1_0) => 20,
            SocketCapability::HTTPS(TLSVersion::TLSv1_1) => 21,
            SocketCapability::HTTPS(TLSVersion::TLSv1_2) => 22,
            SocketCapability::HTTPS(TLSVersion::TLSv1_3) => 23,
        }
    }
}
