mod client;
mod connection;
mod connection_apis;
mod id;
mod macros;

use wasm_bindgen::prelude::*;

#[derive(Eq, PartialOrd, Ord, PartialEq, Copy, Clone, Debug)]
pub enum TLSVersion {
    TLSv1_0 = 0,
    TLSv1_1 = 1,
    TLSv1_2 = 2,
    TLSv1_3 = 3,
}

#[derive(Copy, Clone, Debug)]
pub enum SocketCapability {
    TCP,
    HTTP,
    HTTPS(TLSVersion),
}

impl SocketCapability {
    pub fn from_string(s: &str) -> Option<SocketCapability> {
        match s {
            "tcp" => Some(SocketCapability::TCP),
            "http" => Some(SocketCapability::HTTP),
            "https_tls1_0" => Some(SocketCapability::HTTPS(TLSVersion::TLSv1_0)),
            "https_tls1_1" => Some(SocketCapability::HTTPS(TLSVersion::TLSv1_1)),
            "https_tls1_2" => Some(SocketCapability::HTTPS(TLSVersion::TLSv1_2)),
            "https_tls1_3" => Some(SocketCapability::HTTPS(TLSVersion::TLSv1_3)),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            SocketCapability::TCP => "tcp",
            SocketCapability::HTTP => "http",
            SocketCapability::HTTPS(TLSVersion::TLSv1_0) => "https_tls1_0",
            SocketCapability::HTTPS(TLSVersion::TLSv1_1) => "https_tls1_1",
            SocketCapability::HTTPS(TLSVersion::TLSv1_2) => "https_tls1_2",
            SocketCapability::HTTPS(TLSVersion::TLSv1_3) => "https_tls1_3",
        }
        .to_string()
    }
}

/// Get the capabilities of this implementation.
pub fn get_capabilities() -> Vec<SocketCapability> {
    vec![
        SocketCapability::TCP,
        SocketCapability::HTTP,
        SocketCapability::HTTPS(TLSVersion::TLSv1_2),
    ]
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
