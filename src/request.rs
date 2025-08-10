use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    hash::Hash,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum RequestMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Unknown,
}

impl Display for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestMethod::Get => f.write_str("GET"),
            RequestMethod::Post => f.write_str("POST"),
            RequestMethod::Put => f.write_str("PUT"),
            RequestMethod::Delete => f.write_str("DELETE"),
            RequestMethod::Patch => f.write_str("PATCH"),
            RequestMethod::Options => f.write_str("OPTIONS"),
            RequestMethod::Unknown => f.write_str("UNKNOWN"),
        }
    }
}

pub struct Request {
    pub method: RequestMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub path_params: HashMap<String, String>,
}

impl TryFrom<&mut TcpStream> for Request {
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from(value: &mut TcpStream) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(value);
        let mut buffer_string = String::new();

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }

            if line.eq("\r\n") {
                break;
            }
            buffer_string.push_str(line.as_str());
        }

        let buffer_parts = buffer_string.split("\r\n").collect::<Vec<&str>>();
        let mut path = String::new();
        let mut headers = HashMap::new();
        let mut method = String::new();
        let mut body = vec![];

        if let [first_line, headers_slice @ ..] = buffer_parts.as_slice() {
            let first_line_parts = first_line.split(" ").collect::<Vec<&str>>();

            // determine path and method
            if let [raw_method, raw_path, ..] = first_line_parts.as_slice() {
                path.insert_str(0, raw_path.trim());
                method.insert_str(0, raw_method.trim());
            }

            // read the headers
            for header_line in headers_slice {
                if header_line.trim().is_empty() {
                    continue;
                }
                if let [header_name, header_value, ..] =
                    header_line.split(':').collect::<Vec<&str>>().as_slice()
                {
                    headers.insert(
                        header_name.trim().to_string(),
                        header_value.trim().to_string(),
                    );
                }
            }
        }

        let content_length = headers.get("Content-Length");
        if let Some(body_length) = content_length {
            log::trace!("Request body size={}", body_length);
            let buff_length: usize = body_length.parse()?;
            body = vec![0; buff_length];
            reader.read_exact(&mut body)?;
        }
        Ok(Self {
            method: match method.as_str() {
                "GET" => RequestMethod::Get,
                "POST" => RequestMethod::Post,
                "PUT" => RequestMethod::Put,
                "DELETE" => RequestMethod::Delete,
                "PATH" => RequestMethod::Patch,
                "OPTIONS" => RequestMethod::Options,
                _ => RequestMethod::Unknown,
            },
            path,
            headers,
            body: body.to_vec(),
            path_params: HashMap::new(),
        })
    }
}
