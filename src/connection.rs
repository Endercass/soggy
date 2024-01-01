use std::{
    borrow::Borrow,
    error,
    fmt::{self, Error},
    rc::Rc,
    str::Split,
};

use wasm_bindgen::prelude::*;

use wasm_bindgen_futures::js_sys::{self, Array, ArrayBuffer, Math::log};
use web_sys::{
    js_sys::Uint8Array, AddEventListenerOptions, Blob, BlobPropertyBag, MessageEvent, WebSocket,
};

use crate::{client::Client, console_log, http, id::ConnId, SocketCapability};

#[derive(Clone, Debug)]
pub struct Connection {
    /// WebSocket connection
    socket: WebSocket,
    /// Address of this connection (not the client)
    addr: String,
    /// Protocol used for this connection
    protocol: SocketCapability,
    /// ID of this connection
    id: ConnId,
}

pub struct SocketAddr;

impl SocketAddr {
    pub fn split_addr(protocol: SocketCapability, addr: String) -> Option<String> {
        let mut addr = addr;
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
    message: String,
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

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct HttpHeader {
    /// Header name
    pub(crate) name: String,
    /// Header value
    pub(crate) value: String,
}

#[wasm_bindgen]
impl HttpHeader {
    /// Create a new HTTP header.
    ///
    /// # Arguments
    ///
    /// * `name` - Header name
    /// * `value` - Header value
    #[wasm_bindgen]
    pub fn of(name: String, value: String) -> Self {
        Self { name, value }
    }

    /// Get the header name.
    /// # Returns
    /// The header name.
    #[wasm_bindgen]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Get the header value.
    /// # Returns
    /// The header value.
    #[wasm_bindgen]
    pub fn get_value(&self) -> String {
        self.value.clone()
    }
}

#[wasm_bindgen]
pub struct HttpConnectionRequest {
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
impl HttpConnectionRequest {
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
pub struct HttpConnectionResponse {
    /// Response code
    code: u16,
    /// Response headers
    headers: Vec<HttpHeader>,
    /// Response body
    body: Option<Vec<u8>>,
}

#[wasm_bindgen]
impl HttpConnectionResponse {
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
pub struct HttpConnectionApi {
    /// Connection to create API for
    connection: Connection,
}

impl HttpConnectionApi {
    /// Create a new API instance for the given connection.
    ///
    /// # Arguments
    ///
    /// * `connection` - Connection to create API for
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }
}

#[wasm_bindgen]
impl HttpConnectionApi {
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
        data: HttpConnectionRequest,
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
        console_log!("Sending request: {:?}", req);

        // let callback_rc = Rc::new(callback);

        let message_callback: JsValue = Closure::once_into_js(move |evt: MessageEvent| {
            let buffer = evt.data().dyn_into::<ArrayBuffer>().unwrap_throw();
            let str = String::from_utf8(Uint8Array::new(&buffer).to_vec()).unwrap_throw();

            let mut lines = str.split("\r\n");

            let this = JsValue::null();

            let code: u16 = lines
                .nth(0)
                .unwrap_throw()
                .split(' ')
                .nth(1)
                .unwrap_throw()
                .parse()
                .unwrap_throw();

            let mut headers: Vec<HttpHeader> = Vec::new();

            lines
                .clone()
                .take_while(|line| !line.is_empty())
                .for_each(|line| {
                    let mut split = line.split(':');
                    let name = split.next().unwrap_throw().to_string();
                    let value = split.next().unwrap_throw().to_string();
                    headers.push(HttpHeader::of(name, value));
                });

            let mut body: Option<Vec<u8>> = None;

            lines
                .skip_while(|line| !line.is_empty())
                .skip(1)
                .for_each(|line| {
                    if body.is_none() {
                        body = Some(Vec::new());
                    }
                    body.as_mut()
                        .unwrap_throw()
                        .extend_from_slice(line.as_bytes());
                });

            let response = HttpConnectionResponse::new(code, headers, body);

            console_log!("Received response with code: {}", code,);

            callback
                .call1(&this, &JsValue::from(response))
                .unwrap_throw();
        });

        let _ = self
            .connection
            .socket
            .add_event_listener_with_callback_and_add_event_listener_options(
                "message",
                message_callback.as_ref().unchecked_ref(),
                AddEventListenerOptions::new().once(true),
            )
            .unwrap_throw();

        let _ = self
            .connection
            .socket
            .send_with_u8_array(&req)
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
