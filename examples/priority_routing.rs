// The routing is flexible enough to allow specific routes to be handled even though
// there are path parameters that would match anything else.
// To mix specific routes with routes that look identical but match on path params
// first define the specific routes and then add the route that matches on path params.
use simple_http_server::{App, Result, request::Request, response::Response};

fn echo(request: &Request, response: Response) -> Response {
    let param = request.path_params.get("fruit");
    if let Some(fruit) = param {
        response
            .set_body(format!("You should eat fruits everyday. {} is delicious.", fruit).as_str())
            .set_status(200)
    } else {
        response.set_body("no value?").set_status(500)
    }
}

fn apple(_request: &Request, response: Response) -> Response {
    response
        .set_body("An apple a day keeps the doctor away!")
        .set_status(200)
}

pub fn main() -> Result<()> {
    App::new()
        .set_listen_ip("0.0.0.0")
        .set_port(8080)
        .get("/fruit/apple", apple)
        .get("/fruit/:fruit", echo)
        .run()
}
