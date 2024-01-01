/// A macro to generate a raw HTTP request as a base64 string.
/// # Arguments
/// * `method` - Request method
/// * `path` - Request path
/// * `headers` - Request headers
/// * `body` - Request body
#[macro_export]
macro_rules! http {
    ($method:expr, $path:expr, $headers:expr, $body:expr) => {{
        let mut request = format!("{} {} HTTP/1.1\r\n", $method, $path);

        let headers: Vec<crate::connection::HttpHeader> = $headers;

        let body: Vec<u8> = $body;

        for header in headers {
            request.push_str(&format!("{}: {}\r\n", header.name, header.value));
        }

        request.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
        request.push_str(&String::from_utf8_lossy(body.as_slice()));

        request.into_bytes()
    }};
    ($method:expr, $path:expr, $headers:expr) => {{
        let mut request = format!("{} {} HTTP/1.1\r\n", $method, $path);

        let headers: Vec<crate::connection::HttpHeader> = $headers;

        for header in headers {
            request.push_str(&format!("{}: {}\r\n", header.name, header.value));
        }

        request.push_str("\r\n");

        request.into_bytes()
    }};
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (crate::log(&format_args!($($t)*).to_string()))
}
