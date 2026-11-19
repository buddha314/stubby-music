//! `cargo run -p stubby_rules_server [-- --port 7878 --dir assets/recipes --root .]`: serve the
//! rules designer (and its lore guide, `/lore`) on 127.0.0.1 only. Open the printed address in a browser.
use std::path::PathBuf;
use stubby_rules_server::Designer;

fn main() -> std::process::ExitCode {
    let mut port: u16 = 7878;
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut dir = root.join("assets/recipes");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--port", Some(value)) => match value.parse() {
                Ok(value) => port = value,
                Err(_) => return fail(&format!("--port {value} is not a port")),
            },
            ("--dir", Some(value)) => dir = PathBuf::from(value),
            ("--root", Some(value)) => root = PathBuf::from(value),
            _ => {
                return fail(
                    "usage: stubby_rules_server [--port N] [--dir RULES_DIR] [--root REPO]",
                );
            }
        }
    }
    let mut designer = match Designer::open(&dir) {
        Ok(designer) => designer.with_root(&root),
        Err(error) => return fail(&format!("{}: {error}", dir.display())),
    };
    // Loopback only: the designer writes the rules directory.
    let server = match tiny_http::Server::http(("127.0.0.1", port)) {
        Ok(server) => server,
        Err(error) => return fail(&format!("127.0.0.1:{port}: {error}")),
    };
    println!(
        "Rules designer: http://127.0.0.1:{port}/  ({})",
        dir.display()
    );
    for mut request in server.incoming_requests() {
        let mut body = String::new();
        if let Err(error) = request.as_reader().read_to_string(&mut body) {
            eprintln!("request body: {error}");
            continue;
        }
        let reply = designer.handle(request.method().as_str(), request.url(), &body);
        let response = tiny_http::Response::from_data(reply.body)
            .with_status_code(reply.status)
            .with_header(
                tiny_http::Header::from_bytes("Content-Type", reply.content_type)
                    .expect("static header"),
            );
        if let Err(error) = request.respond(response) {
            eprintln!("response: {error}");
        }
    }
    std::process::ExitCode::SUCCESS
}

fn fail(message: &str) -> std::process::ExitCode {
    eprintln!("{message}");
    std::process::ExitCode::FAILURE
}
