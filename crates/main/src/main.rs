use std::{
    env,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
    sync::Arc,
};

use log::trace;
use rthttp_request::Request;
use rthttp_response::Response;
use rthttp_router::{App, Result};

fn index_handler(_request: &Request, response: Response) -> Response {
    response.set_status(200)
}

fn echo_handler(request: &Request, response: Response) -> Response {
    response
        .set_status(200)
        .set_body(request.path_params.get("fruit").unwrap())
}

fn user_agent(request: &Request, response: Response) -> Response {
    let user_agent = request.headers.get("User-Agent");
    let mut response = response.set_status(200);
    if let Some(user_agent) = user_agent {
        response = response.set_body(user_agent);
    }

    response
}

pub fn main() -> Result<()> {
    env_logger::init();
    let arguments: Vec<String> = env::args().collect();
    let mut static_folder: Arc<Option<String>> = Arc::new(None);

    for i in 1..arguments.len() {
        if arguments[i].eq("--directory") {
            static_folder = Arc::from(Some(arguments[i + 1].clone()));
        }
    }
    drop(arguments);

    let post_static_folder = static_folder.clone();

    let app = App::new()
        .set_listen_ip("127.0.0.1")
        .set_port(4221)
        .index(index_handler)
        .get("/echo/:fruit", echo_handler)
        .get("user-agent", user_agent)
        .get("/files/:filename", move |request, response| {
            if let Some(folder) = &*static_folder {
                let file_path =
                    Path::new(folder).join(request.path_params.get("filename").unwrap());
                if !file_path.exists() {
                    return response
                        .set_body("Cannot read requested file")
                        .set_status(404);
                }

                let file_content = read_to_string(file_path);

                if let Err(err) = file_content {
                    log::error!("{err}");
                    return response.set_status(500).set_body(format!("{err}").as_str());
                }

                let file_content = file_content.unwrap();
                return response
                    .set_body(&file_content)
                    .set_status(200)
                    .add_header("Content-Type", "application/octet-stream");
            }

            response
        })
        .post("/files/:filename", move |request, response| {
            if let Some(static_folder) = &*post_static_folder {
                let path =
                    Path::new(static_folder).join(request.path_params.get("filename").unwrap());
                trace!("POST/files/:filename filepath={}", path.to_string_lossy());
                let file = File::create(path);
                trace!("POST/files/:filename createFile=yes");

                if let Err(create_error) = file {
                    log::error!("{create_error}");
                    return response
                        .set_status(500)
                        .set_body(format!("{create_error}").as_str());
                }
                let mut file = file.unwrap();

                trace!("POST/files/:filename contentSize={}", request.body.len());
                trace!(
                    "POST/files/:filename content={}",
                    std::str::from_utf8(request.body.as_slice()).unwrap()
                );
                if let Err(err) = file.write_all(&request.body) {
                    return response
                        .set_status(500)
                        .set_body(format!("Could not write: {err}").as_str());
                }
                let _ = file.flush();
                return response.set_status(201);
            }
            response
        });

    app.run()
}
