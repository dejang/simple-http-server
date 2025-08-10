# Simple HTTP Server
A very basic Web Server library (less than 1 MB) that could be used for simple scenarios like local web development or in an embedded application that does not need all the bells and whistles frameworks bring.

Only one dependency and that is for gzip compression on responses.
This project is more of a learning experience than something that's meant for real-world use. Nevertheless, as basic as it is, it is suprising how far it can go.

## Features
- supports path parameters in the form of `/user/:id`
- supports priority routing when it overlaps with path parameters. For instance a specific route defined as `/user/superadmin` could be handled by a different handler than `/user/:id`. Check the example folder for more details.
- has a simple API for static folder mapping which allows serving static content, same as you'd expect from any other static server for local web development.
- supports file uploads. Check the `file_upload` example.

## Usage

```
use std::path::PathBuf;

use simple_http_server::{App, Result};

pub fn main() -> Result<()> {
    let folder_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/static");
    App::new()
        .set_listen_ip("0.0.0.0")
        .set_port(8080)
        .static_folder("/static", folder_path.as_path())
        .run()
}
```

Examples can be run using `cargo run --example <example_file>`.

## Design
There are 4 parts to this library:
- Request: responsible for transforming an incoming raw TcpStream to a Request object that we can use throughout the rest of the codebase.
- Response: responsible for manipulating the returning response to the network and provides an API to the user with convienience methods for adding headers, setting the body and status code of the response
- Route Matcher: a Trie based matcher that allows complex routes to be defined and is also the main reason priority routes feature can be achieved, as opposed to a Regex based matcher which could stumble in such scenarios.
- The App API module: exposes configuration methods and allows defining routes. Internally, it uses a thread for every incoming request. There is a great deal of improvement that can be made here to achieve maximum throughput if instead of running one handler per thread it runs async tasks on each thread. But this would mean bringing in tokio library and that defeats the purpose of keeping it simple and light. A light async Executor could be an interesting exercise.

## Limitations
- support query params to be implemented
- form posts not currently supported (contributions welcomed!)
- bring your own JSON serializer/deserializer
- no traits for responses, similar to Axum's IntoResponse<T>
- no SSL support
- no HTTP 2 support

## Final note
Limitations often push creativity boundaries and I hope this shows how much can be achieved with very little code. I've built this while working on a [codercraft](https://codecrafters.io/) challenge. It's been fun and I suggest to anyone wanting to learn more about Rust to try their challenges.
