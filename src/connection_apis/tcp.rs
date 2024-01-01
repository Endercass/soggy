use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::{self, ArrayBuffer, Uint8Array};
use web_sys::{AddEventListenerOptions, MessageEvent};

use crate::{
    connection::{Connection, ConnectionError},
};

#[wasm_bindgen]
pub struct TcpConnectionRequest {
    /// Request body
    body: Vec<u8>,
}

#[wasm_bindgen]
impl TcpConnectionRequest {
    /// Create a new TCP request.
    ///
    /// # Arguments
    ///
    /// * `body` - Request body
    #[wasm_bindgen(constructor)]
    pub fn new(body: Vec<u8>) -> Self {
        Self { body }
    }
}

#[wasm_bindgen]
pub struct TcpConnectionResponse {
    /// Response body
    body: Vec<u8>,
}

#[wasm_bindgen]
impl TcpConnectionResponse {
    /// Create a new HTTP response.
    ///
    /// # Arguments
    ///
    /// * `code` - Response code
    /// * `headers` - Response headers
    /// * `body` - Response body
    #[wasm_bindgen(constructor)]
    pub fn new(body: Vec<u8>) -> Self {
        Self { body }
    }
    /// Get the response body.
    #[wasm_bindgen]
    pub fn get_body(&self) -> Vec<u8> {
        return self.body.clone();
    }
}

#[wasm_bindgen]
pub struct TcpConnectionApi {
    /// Connection to create API for
    connection: Connection,
}

impl TcpConnectionApi {
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
impl TcpConnectionApi {
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
        data: TcpConnectionRequest,
        callback: js_sys::Function,
    ) -> Result<(), ConnectionError> {
        if (self.connection.socket.ready_state() as u16) != 1 {
            return Err(ConnectionError {
                message: "Connection is not open".to_string(),
            });
        }

        let message_callback: JsValue = Closure::once_into_js(move |evt: MessageEvent| {
            let buffer = evt.data().dyn_into::<ArrayBuffer>().unwrap_throw();
            let vec = Uint8Array::new(&buffer).to_vec();

            let this = JsValue::null();

            let response = TcpConnectionResponse::new(vec);

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
            .send_with_u8_array(&data.body)
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
