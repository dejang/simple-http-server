use simple_http_server::{App, Result, request::Request, response::Response};

fn echo(request: &Request, response: Response) -> Response {
    let echo_value = request.path_params.get("echo");
    if let Some(value) = echo_value {
        response.set_body(value).set_status(200)
    } else {
        response.set_body("no value?").set_status(500)
    }
}

pub fn main() -> Result<()> {
    App::new()
        .set_listen_ip("0.0.0.0")
        .set_port(8080)
        .get("/:echo", echo)
        .run()
}
