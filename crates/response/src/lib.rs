use std::collections::HashMap;

#[derive(Default)]
pub struct Response {
    headers: HashMap<String, String>,
    body: Vec<u8>,
    status_code: u16,
}

impl Response {
    pub fn new() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".into(), "text/plain".into());
        Self {
            headers,
            body: vec![],
            status_code: 0,
        }
    }

    pub fn add_header(mut self, header_name: &str, header_value: &str) -> Self {
        self.headers
            .insert(header_name.to_owned(), header_value.to_owned());
        self
    }

    pub fn set_body(mut self, body: &str) -> Self {
        self.body = body.as_bytes().to_vec();
        self.headers
            .insert("Content-Length".to_owned(), self.body.len().to_string());
        self
    }

    pub fn set_body_bytes(mut self, body: &[u8]) -> Self {
        self.body = body.to_vec();
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }

    pub fn get_body(&self) -> &Vec<u8> {
        &self.body
    }

    pub fn set_status(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }

    pub fn build(self) -> Vec<u8> {
        let status_string = match self.status_code {
            200 => "200 OK",
            201 => "201 Created",
            401 => "401 Access Denied",
            404 => "404 Not Found",
            500 => "500 Internal Server Error",
            _ => "500 Internal Server Error",
        };
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
            .join("\r\n");

        [
            format!("HTTP/1.1 {}\r\n{}\r\n\r\n", status_string, headers).as_bytes(),
            self.body.as_slice(),
        ]
        .concat()
    }
}
