use std::{
    collections::HashMap,
    error::Error,
    io::Write,
    net::TcpStream,
    sync::{Arc, RwLock},
    thread::{self},
};

use log::trace;
use node::{Node, PathParams};
use rthttp_request::{Request, RequestMethod};
use rthttp_response::Response;

use flate2::{write::GzEncoder, Compression};
use std::net::TcpListener;
mod node;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;

pub type RequestHandler = dyn Fn(&Request, Response) -> Response + Send + Sync + 'static;

pub struct Router {
    pub(crate) routes: HashMap<RequestMethod, HashMap<String, Box<RequestHandler>>>,
    pub(crate) roots: HashMap<RequestMethod, Node>,
    pub(crate) index_handler: Box<RequestHandler>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
            roots: HashMap::new(),
            index_handler: Box::new(|_, r| r),
        }
    }

    pub fn index<F>(&mut self, handler: F)
    where
        F: Fn(&Request, Response) -> Response + Send + Sync + 'static,
    {
        self.index_handler = Box::new(handler);
    }

    pub fn route<F>(&mut self, method: RequestMethod, url_pattern: &str, handler: F)
    where
        F: Fn(&Request, Response) -> Response + Send + Sync + 'static,
    {
        let node = self.roots.entry(method.clone()).or_default();
        if node.is_match(url_pattern).is_some() {
            panic!("A handler has already been defined for this url pattern");
        }

        self.routes
            .entry(method)
            .or_default()
            .insert(url_pattern.to_string(), Box::new(handler));
        node.append(url_pattern);
    }

    pub(crate) fn get_handler(
        &self,
        method: &RequestMethod,
        url: &str,
    ) -> Option<(&RequestHandler, PathParams)> {
        if url.eq("/") {
            return Some((&self.index_handler, PathParams::new()));
        }
        let node = self.roots.get(method)?;
        if let Some((url_pattern, path_params)) = node.is_match(url) {
            return Some((
                self.routes.get(method).unwrap().get(&url_pattern).unwrap(),
                path_params,
            ));
        }
        None
    }
}

pub struct App {
    router: Arc<RwLock<Router>>,
    port: u16,
    listen_ip: String,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            router: Arc::new(RwLock::new(Router::new())),
            port: 0,
            listen_ip: "0.0.0.0".into(),
        }
    }

    pub(crate) fn is_compression_supported(compression_str: &str) -> Option<String> {
        let str_parts = compression_str
            .split(",")
            .map(|v| v.trim())
            .collect::<Vec<&str>>();
        for compression_scheme in str_parts {
            if compression_scheme.eq_ignore_ascii_case("gzip") {
                return Some(compression_scheme.to_string());
            }
        }
        None
    }

    pub(crate) fn compress(input: &[u8], _compression_type: &str) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

        encoder.write_all(input)?;
        encoder.flush()?;
        let buffer = encoder.finish()?;
        Ok(buffer)
    }

    pub(crate) fn get_response<F>(request: Request, handler: F) -> Response
    where
        F: Fn(&Request, Response) -> Response + Send + Sync,
    {
        let compression = match request.headers.get("Accept-Encoding") {
            Some(v) => App::is_compression_supported(v),
            None => None,
        };

        let response = handler(&request, Response::new());
        match compression {
            Some(v) => {
                if let Ok(compressed) = App::compress(response.get_body().as_slice(), &v) {
                    return response
                        .add_header("Content-Encoding", "gzip")
                        .set_body_bytes(&compressed);
                }
                response
            }
            None => response,
        }
    }

    fn request_handler(&self, mut stream: TcpStream) -> Result<()> {
        let router = self.router.clone();
        thread::spawn(move || {
            let mut request = Request::from_tcp_stream(&mut stream).unwrap();

            let path = request.path.as_str();
            let response = if let Some((handler, path_params)) =
                router.read().unwrap().get_handler(&request.method, path)
            {
                request.path_params = path_params;
                App::get_response(request, handler)
            } else {
                Response::new().set_status(404)
            };
            let _ = stream.write_all(response.build().as_slice());
        });
        Ok(())
    }

    pub fn run(self) -> Result<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.listen_ip, self.port))
            .expect("Failed to create TCP Socket");

        for stream in listener.incoming() {
            let stream = stream?;
            self.request_handler(stream)?;
        }
        Ok(())
    }

    pub fn set_listen_ip(mut self, ip: &str) -> Self {
        self.listen_ip = ip.to_string();
        self
    }

    pub fn set_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn index<F>(self, handler: F) -> Self
    where
        F: Fn(&Request, Response) -> Response + Send + Sync + 'static,
    {
        self.router.write().unwrap().index(handler);
        self
    }

    pub fn get<F>(self, path: &str, handler: F) -> Self
    where
        F: Fn(&Request, Response) -> Response + Send + Sync + 'static,
    {
        self.router
            .write()
            .unwrap()
            .route(RequestMethod::Get, path, handler);
        self
    }

    pub fn post<F>(self, path: &str, handler: F) -> Self
    where
        F: Fn(&Request, Response) -> Response + Send + Sync + 'static,
    {
        self.router
            .write()
            .unwrap()
            .route(RequestMethod::Post, path, handler);
        self
    }

    pub fn put<F>(self, path: &str, handler: F) -> Self
    where
        F: Fn(&Request, Response) -> Response + Send + Sync + 'static,
    {
        self.router
            .write()
            .unwrap()
            .route(RequestMethod::Put, path, handler);
        self
    }

    pub fn delete<F>(self, path: &str, handler: F) -> Self
    where
        F: Fn(&Request, Response) -> Response + Send + Sync + 'static,
    {
        self.router
            .write()
            .unwrap()
            .route(RequestMethod::Delete, path, handler);
        self
    }

    pub fn patch<F>(self, path: &str, handler: F) -> Self
    where
        F: Fn(&Request, Response) -> Response + Send + Sync + 'static,
    {
        self.router
            .write()
            .unwrap()
            .route(RequestMethod::Patch, path, handler);
        self
    }

    pub fn options<F>(self, path: &str, handler: F) -> Self
    where
        F: Fn(&Request, Response) -> Response + Send + Sync + 'static,
    {
        self.router
            .write()
            .unwrap()
            .route(RequestMethod::Options, path, handler);
        self
    }
}
