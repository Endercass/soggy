use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};

use rustls::{
    client::ClientConnectionData,
    server,
    version::{TLS12, TLS13},
    ClientConfig, ClientConnection, RootCertStore,
};
use rustls_pki_types::{DnsName, IpAddr, ServerName};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::{self, ArrayBuffer, Uint8Array};
use web_sys::{AddEventListenerOptions, MessageEvent};

use crate::{
    connection::{Connection, ConnectionError},
    console_log, http, SocketCapability, TLSVersion,
};

use super::http::HttpHeader;

#[wasm_bindgen]
pub struct HttpsConnectionRequest {
    /// Request method
    method: String,
    /// Request path
    path: String,
    /// Request headers
    headers: Vec<HttpHeader>,
    /// Request body
    body: Option<Vec<u8>>,
}

#[wasm_bindgen]
impl HttpsConnectionRequest {
    /// Create a new HTTP request.
    ///
    /// # Arguments
    ///
    /// * `method` - Request method
    /// * `headers` - Request headers
    /// * `body` - Request body
    #[wasm_bindgen(constructor)]
    pub fn new(
        method: String,
        path: String,
        headers: Vec<HttpHeader>,
        body: Option<Vec<u8>>,
    ) -> Self {
        Self {
            method,
            path,
            headers,
            body,
        }
    }
}

#[wasm_bindgen]
pub struct HttpsConnectionResponse {
    /// Response code
    code: u16,
    /// Response headers
    headers: Vec<HttpHeader>,
    /// Response body
    body: Option<Vec<u8>>,
}

#[wasm_bindgen]
impl HttpsConnectionResponse {
    /// Create a new HTTP response.
    ///
    /// # Arguments
    ///
    /// * `code` - Response code
    /// * `headers` - Response headers
    /// * `body` - Response body
    #[wasm_bindgen(constructor)]
    pub fn new(code: u16, headers: Vec<HttpHeader>, body: Option<Vec<u8>>) -> Self {
        Self {
            code,
            headers,
            body,
        }
    }

    /// Get the response code.
    #[wasm_bindgen]
    pub fn get_code(&self) -> u16 {
        self.code
    }

    /// Get the response headers.
    #[wasm_bindgen]
    pub fn get_headers(&self) -> Vec<HttpHeader> {
        self.headers.clone()
    }

    /// Get the response body.
    #[wasm_bindgen]
    pub fn get_body(&self) -> Option<Vec<u8>> {
        return self.body.clone();
    }
}

#[wasm_bindgen]
pub struct HttpsConnectionApi {
    /// Connection to create API for
    connection: Connection,
    /// TLS client config
    config: Arc<ClientConfig>,
    /// TLS server name
    server_name: ServerName<'static>,
}

impl HttpsConnectionApi {
    /// Create a new API instance for the given connection.
    ///
    /// # Arguments
    ///
    /// * `connection` - Connection to create API for
    pub fn new(connection: Connection) -> Self {
        let root_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };

        let protocol_version = match connection.protocol {
            SocketCapability::HTTPS(TLSVersion::TLSv1_2) => &TLS12,
            SocketCapability::HTTPS(TLSVersion::TLSv1_3) => &TLS13,
            _ => panic!("Invalid protocol version"),
        };

        let config = Arc::new(
            ClientConfig::builder_with_protocol_versions(&[protocol_version])
                .with_root_certificates(root_store)
                .with_no_client_auth(),
        );

        // Determine if the server name is an IP address or a domain name

        let addr: String = connection
            .addr
            .clone()
            .split(':')
            .next()
            .unwrap_throw()
            .to_string();

        console_log!("Connecting to {}", addr);

        let ip_regex = regex::Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$|^([0-9A-Fa-f]{0,4}:){2,7}([0-9A-Fa-f]{1,4}$|((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(\.|$)){4})$").unwrap_throw();

        let server_name: ServerName = if ip_regex.is_match(&addr) {
            ServerName::IpAddress(IpAddr::try_from(addr.as_str()).unwrap_throw())
        } else {
            ServerName::DnsName(DnsName::try_from(addr).unwrap_throw())
        };

        Self {
            connection,
            config,
            server_name,
        }
    }
}

#[wasm_bindgen]
impl HttpsConnectionApi {
    #[wasm_bindgen]
    /// Get the address of this connection.
    pub fn get_addr(&self) -> String {
        self.connection.addr.clone()
    }

    /// Send data to this connection.
    ///
    /// # Arguments
    ///
    /// * `data` - Data to send to this connection. The type of this data depends on the implementation.
    /// * `callback` - Callback to call when data is received from this connection.
    ///
    /// # Returns
    ///
    /// The function returns a Result containing a void, or an error depending on the success of the send.
    /// * `ConnectionError` - Error that occurred while sending data to this connection.
    #[wasm_bindgen]
    pub fn send(
        &self,
        data: HttpsConnectionRequest,
        callback: js_sys::Function,
    ) -> Result<(), ConnectionError> {
        if (self.connection.socket.ready_state() as u16) != 1 {
            return Err(ConnectionError {
                message: "Connection is not open".to_string(),
            });
        }
        let req = if let Some(body) = data.body {
            http!(data.method, data.path, data.headers, body.to_vec())
        } else {
            http!(data.method, data.path, data.headers)
        };

        let mut conn = rustls::ClientConnection::new(self.config.clone(), self.server_name.clone())
            .unwrap_throw();

        conn.writer().write_all(&req).unwrap_throw();

        let mut tls = Vec::new();
        conn.write_tls(&mut tls).unwrap_throw();

        let _ = conn.process_new_packets().unwrap_throw();

        let cb_conn = Arc::new(Mutex::new(conn));

        let encoded_response: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

        let response_code: Arc<Mutex<u16>> = Arc::new(Mutex::new(0u16));

        let response_headers: Arc<Mutex<Vec<HttpHeader>>> = Arc::new(Mutex::new(Vec::new()));

        let response_body: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

        let content_length: Arc<Mutex<usize>> = Arc::new(Mutex::new(0usize));

        let message_callback: Closure<dyn Fn(MessageEvent)> =
            Closure::wrap(Box::new(move |evt: MessageEvent| {
                let buffer = evt.data().dyn_into::<ArrayBuffer>().unwrap_throw();
                let tls = Uint8Array::new(&buffer).to_vec();

                let mut encoded_response = encoded_response.lock().unwrap_throw();

                let mut cb_conn = cb_conn.lock().unwrap_throw();

                console_log!("Received TLS: {:?}", tls);

                (*encoded_response).extend_from_slice(&tls);

                console_log!("cumulative TLS: {:?}", *encoded_response);
                console_log!("TLS len: {}", encoded_response.len());

                if encoded_response.len() == 4011 {
                    console_log!("TESTING: example tls complete");
                    cb_conn
                        .read_tls(&mut encoded_response.as_slice())
                        .unwrap_throw();
                    let mut vec: Vec<u8> = Vec::new();

                    // cb_conn.reader().read_to_end(&mut vec).unwrap_throw();

                    console_log!("Received response: {:?}", cb_conn.);
                }

                drop(encoded_response);
                drop(cb_conn)

                // cb_conn.read_tls(&mut tls.as_slice()).unwrap_throw();
                // let _ = cb_conn.process_new_packets().unwrap_throw();

                // let mut vec: Vec<u8> = Vec::new();

                // cb_conn.reader().read_to_end(&mut vec).unwrap_throw();

                // console_log!(
                //     "Received response: {}",
                //     String::from_utf8(vec.clone()).unwrap_throw()
                // );
            }));

        let _ = self
            .connection
            .socket
            .add_event_listener_with_callback_and_add_event_listener_options(
                "message",
                message_callback.as_ref().unchecked_ref(),
                AddEventListenerOptions::new().once(false),
            )
            .unwrap_throw();

        message_callback.forget();

        let _ = self
            .connection
            .socket
            .send_with_u8_array(&tls)
            .unwrap_throw();

        Ok(())
    }

    /// Ping this connection.
    ///
    /// # Returns
    ///
    /// The function returns a void, or an error depending on the success of the ping.
    #[wasm_bindgen]
    pub fn ping(&self) -> Result<(), ConnectionError> {
        Ok(())
    }

    /// Close this connection.
    pub fn close(&self) {
        let _ = self.connection.socket.close();
    }
}
