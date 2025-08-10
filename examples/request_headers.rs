use simple_http_server::{App, Result, request::Request, response::Response};

fn user_agent(request: &Request, response: Response) -> Response {
    let user_agent = request.headers.get("User-Agent");
    let mut response = response.set_status(200);
    if let Some(user_agent) = user_agent {
        response = response.set_body(user_agent);
    }

    response
}

pub fn main() -> Result<()> {
    App::new()
        .set_listen_ip("0.0.0.0")
        .set_port(8080)
        .get("/", user_agent)
        .run()
}
