use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::{self, ArrayBuffer, Uint8Array};
use web_sys::{AddEventListenerOptions, MessageEvent};

use crate::{
    connection::{Connection, ConnectionError},
    console_log, http,
};

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
