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
