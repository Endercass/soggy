use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;

use crate::{
    connection::{Connection, SocketAddr},
    connection_apis::{http::HttpConnectionApi, tcp::TcpConnectionApi},
    get_capabilities,
    id::ConnIdFactory,
    SocketCapability, TLSVersion,
};

#[wasm_bindgen]
pub struct Client {
    /// Factory for connection IDs.
    factory: ConnIdFactory,
    /// Base socket address of this client.
    addr: String,
    /// Connections belonging to this client.
    connections: Vec<Connection>,
    /// Capabilities of this client.
    capabilities: Vec<SocketCapability>,
}

#[wasm_bindgen]
impl Client {
    /// Create a new client using the given socket address, and the default capabilities.
    #[wasm_bindgen(constructor)]
    pub fn new(addr: String) -> Self {
        Client {
            factory: ConnIdFactory::new(),
            addr,
            connections: Vec::new(),
            capabilities: get_capabilities(),
        }
    }
    /// Create a new client using the given socket address, and the given capabilities.
    #[wasm_bindgen]
    pub fn new_with_capabilities(addr: String, capabilities: Vec<String>) -> Self {
        let capabilities: Vec<SocketCapability> = capabilities
            .iter()
            .filter_map(|s| SocketCapability::from_string(s.to_lowercase().as_str()))
            .collect();
        Client {
            factory: ConnIdFactory::new(),
            addr,
            connections: Vec::new(),
            capabilities,
        }
    }
    /// Get the base wsproxy url of this client.
    #[wasm_bindgen]
    pub fn get_addr(&self) -> String {
        self.addr.clone()
    }
    /// Get the capabilities of this client.
    #[wasm_bindgen]
    pub fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.iter().map(|c| c.to_string()).collect()
    }
    /// Get the capabilities of this implementation.
    #[wasm_bindgen]
    pub fn get_impl_capabilities() -> Vec<String> {
        crate::get_capabilities()
            .iter()
            .map(|c| c.to_string())
            .collect()
    }
    /// Create a new http connection to the given address.
    /// # Arguments
    /// * `addr` - Address to connect to
    #[wasm_bindgen]
    pub fn create_http_connection(&mut self, addr: String) -> Option<HttpConnectionApi> {
        let protocol = SocketCapability::HTTP;
        let id = self.factory.generate(protocol);
        let addr = SocketAddr::split_addr(protocol, addr).unwrap();
        let connection = Connection::new(self, protocol, addr, id).unwrap();
        self.connections.push(connection.clone());
        Some(HttpConnectionApi::new(connection))
    }

    /// Create a new http connection to the given address with an onready callback.
    /// # Arguments
    /// * `addr` - Address to connect to
    /// * `callback` - Callback to call when the connection is ready
    #[wasm_bindgen]
    pub fn create_http_connection_with_onready(
        &mut self,
        addr: String,
        callback: js_sys::Function,
    ) -> Option<HttpConnectionApi> {
        let protocol = SocketCapability::HTTP;
        let id = self.factory.generate(protocol);
        let addr = SocketAddr::split_addr(protocol, addr).unwrap();
        let connection = Connection::new(self, protocol, addr, id).unwrap();
        connection.set_onready(callback, None);
        self.connections.push(connection.clone());
        Some(HttpConnectionApi::new(connection))
    }

    /// Get a http connection API for the given connection.
    #[wasm_bindgen]
    pub fn get_http_connection_api(&self, id: u64) -> HttpConnectionApi {
        self.connections
            .iter()
            .find(|c| Into::<u64>::into(c.get_id()) == id)
            .map(|c| HttpConnectionApi::new(c.clone()))
            .unwrap()
    }

    /// Create a new http connection to the given address.
    /// # Arguments
    /// * `addr` - Address to connect to
    #[wasm_bindgen]
    pub fn create_tcp_connection(&mut self, addr: String) -> Option<TcpConnectionApi> {
        let protocol = SocketCapability::TCP;
        let id = self.factory.generate(protocol);
        let addr = SocketAddr::split_addr(protocol, addr).unwrap();
        let connection = Connection::new(self, protocol, addr, id).unwrap();
        self.connections.push(connection.clone());
        Some(TcpConnectionApi::new(connection))
    }

    /// Create a new http connection to the given address with an onready callback.
    /// # Arguments
    /// * `addr` - Address to connect to
    /// * `callback` - Callback to call when the connection is ready
    #[wasm_bindgen]
    pub fn create_tcp_connection_with_onready(
        &mut self,
        addr: String,
        callback: js_sys::Function,
    ) -> Option<TcpConnectionApi> {
        let protocol = SocketCapability::TCP;
        let id = self.factory.generate(protocol);
        let addr = SocketAddr::split_addr(protocol, addr).unwrap();
        let connection = Connection::new(self, protocol, addr, id).unwrap();
        connection.set_onready(callback, None);
        self.connections.push(connection.clone());
        Some(TcpConnectionApi::new(connection))
    }

    /// Generate a new connection ID.
    #[wasm_bindgen]
    pub fn generate_id(&mut self, conn_type: String) -> u64 {
        let conn_type = SocketCapability::from_string(conn_type.to_lowercase().as_str()).unwrap();
        let id = self.factory.generate(conn_type);
        Into::<u64>::into(id)
    }
}

impl Client {
    /// Get the highest supported TLS version.
    pub fn get_highest_tls_version() -> TLSVersion {
        *get_capabilities()
            .iter()
            .filter_map(|c| match c {
                SocketCapability::HTTPS(v) => Some(v),
                _ => None,
            })
            .max()
            .unwrap()
    }
}
