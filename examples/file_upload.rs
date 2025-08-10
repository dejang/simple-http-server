use std::{fs::File, io::Write, path::PathBuf};

use simple_http_server::{App, Result, request::Request, response::Response};

fn upload_file(request: &Request, response: Response) -> Response {
    let folder_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/gallery/images");
    let file_path = folder_path.join(request.path_params.get("filename").unwrap());
    let file = File::create(file_path);

    if let Err(create_error) = file {
        return response
            .set_status(500)
            .set_body(format!("{create_error}").as_str());
    }
    let mut file = file.unwrap();

    if let Err(err) = file.write_all(&request.body) {
        return response
            .set_status(500)
            .set_body(format!("Could not write: {err}").as_str());
    }
    let _ = file.flush();

    response
        .add_header(
            "Location",
            &format!(
                "/gallery/images/{}",
                request.path_params.get("filename").unwrap()
            ),
        )
        .set_status(201)
}

fn list_files(_request: &Request, response: Response) -> Response {
    let folder_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/gallery/images");
    let files = folder_path.read_dir().unwrap();
    let mut filenames = Vec::new();
    for f in files {
        let file = f.unwrap();
        let filename = file.file_name();
        filenames.push(format!("\"gallery/images/{}\"", filename.to_str().unwrap()));
    }
    let json = format!("[{}]", filenames.join(","));
    response
        .add_header("Content-Type", "application/json")
        .set_body(&json)
        .set_status(200)
}

pub fn main() -> Result<()> {
    App::new()
        .set_listen_ip("0.0.0.0")
        .set_port(8080)
        .static_folder(
            "/gallery",
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("examples/gallery")
                .as_path(),
        )
        .get("/list", list_files)
        .post("/upload/:filename", upload_file)
        .run()
}
