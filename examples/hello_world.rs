use simple_http_server::{App, Result, request::Request, response::Response};

fn say_hello(_request: &Request, response: Response) -> Response {
    response.set_body("Hello, World").set_status(200)
}

pub fn main() -> Result<()> {
    App::new()
        .set_listen_ip("0.0.0.0")
        .set_port(8080)
        .get("/", say_hello)
        .run()
}
