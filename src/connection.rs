use std::{
    error,
    fmt::{self},
};

use wasm_bindgen::prelude::*;

use wasm_bindgen_futures::js_sys::{self};
use web_sys::{
    AddEventListenerOptions, WebSocket,
};

use crate::{client::Client, id::ConnId, SocketCapability};

#[derive(Clone, Debug)]
pub struct Connection {
    /// WebSocket connection
    pub(crate) socket: WebSocket,
    /// Address of this connection (not the client)
    pub(crate) addr: String,
    /// Protocol used for this connection
    pub(crate) protocol: SocketCapability,
    /// ID of this connection
    pub(crate) id: ConnId,
}

pub struct SocketAddr;

impl SocketAddr {
    pub fn split_addr(protocol: SocketCapability, addr: String) -> Option<String> {
        let addr = addr;
        if !addr.contains("://") {
            return Some(addr);
        }
        let mut split = addr.split("://").skip(1); // Skip protocol
        let addr = split.next()?.replace('/', "");

        let default_port = match protocol {
            SocketCapability::TCP => "0",
            SocketCapability::HTTP => "80",
            SocketCapability::HTTPS(_) => "443",
        };

        let mut split = addr.split(':');
        let addr = split.next()?;
        let port = split.next().unwrap_or(default_port);

        return Some(format!("{}:{}", addr, port));
    }
}

impl Connection {
    /// Create a new connection to the given address.
    ///
    /// # Arguments
    ///
    /// * `client` - Reference to client that owns this connection
    /// * `protocol` - Protocol to use for this connection
    /// * `addr` - Address of this connection without protocol (e.g. `tcp://` or `http://`)
    /// * `id` - ID of this connection
    pub fn new(
        client: &Client,
        protocol: SocketCapability,
        addr: String,
        id: ConnId,
    ) -> Result<Self, Box<dyn error::Error>> {
        let base = client.get_addr();

        let socket =
            WebSocket::new_with_str(&format!("{}/{}", base, addr), "binary").unwrap_throw();
        socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
        Ok(Connection {
            socket,
            addr,
            protocol,
            id,
        })
    }

    /// Get the address of this connection.
    pub fn get_addr(&self) -> String {
        self.addr.clone()
    }

    /// Get the protocol of this connection.
    pub fn get_protocol(&self) -> SocketCapability {
        self.protocol
    }

    /// Get the ID of this connection.
    pub fn get_id(&self) -> ConnId {
        self.id
    }

    /// set onready callback
    pub fn set_onready(&self, callback: js_sys::Function, once: Option<bool>) {
        let once = once.unwrap_or(false);
        let _ = self
            .socket
            .add_event_listener_with_callback_and_add_event_listener_options(
                "open",
                &callback,
                AddEventListenerOptions::new().once(once),
            )
            .unwrap_throw();
    }

    /// get onready callback
    pub fn get_onready(&self) -> Option<js_sys::Function> {
        self.socket.onopen()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.socket.close();
    }
}

pub struct ConnectionError {
    /// Error message
    pub message: String,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Connection error: {}", self.message)
    }
}

impl fmt::Debug for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Connection error: {}\n{{ file: {}, line: {} }}",
            self.message,
            file!(),
            line!()
        )
    }
}

impl error::Error for ConnectionError {}

impl Into<JsValue> for ConnectionError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.message)
    }
}
