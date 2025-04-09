use hyperware_process_lib::homepage::add_to_homepage;
use hyperware_process_lib::http::server::{HttpBindingConfig, HttpServer};
use hyperware_process_lib::logging::{info, init_logging, Level};
use hyperware_process_lib::sqlite::Sqlite;
use hyperware_process_lib::{
    await_message, call_init, kiprintln, print_to_terminal, Address, Message,
};

mod db;
mod structs;
use structs::*;
mod chain;
mod helpers;
mod http_handlers;

fn init_http() -> anyhow::Result<HttpServer> {
    let mut http_server = HttpServer::new(5);
    let http_config = HttpBindingConfig::default();

    // REST API
    http_server.bind_http_path("/api/state", http_config.clone())?;
    http_server.bind_http_path("/api/all", http_config.clone())?;
    http_server.bind_http_path("/api/cat", http_config.clone())?;
    http_server.bind_http_path("/api/search", http_config.clone())?;
    http_server.bind_http_path(
        "/api/mcp",
        HttpBindingConfig::new(false, false, false, None),
    )?;
    add_to_homepage("HPN indexer", None, Some("/"), None);
    http_server.serve_ui("ui", vec!["/"], http_config)?;

    Ok(http_server)
}

call_init!(init);
fn init(our: Address) {
    init_logging(Level::DEBUG, Level::INFO, None, None, None).unwrap();
    info!("begin hpn indexer");

    info!("sqlite loaded");

    let mut state = State::load();
    let _http_server = init_http().expect("failed to bind paths");
    let db = db::open_db(&our).unwrap();
    db::check_schema(&db).unwrap();

    let mut pending = chain::start_fetch(&mut state, &db);
    loop {
        if let Err(e) = main(&our, &mut state, &db, &mut pending) {
            // print_to_terminal(1, "fatal error {e}");
            kiprintln!("something wrong at main\n{:#?}", e);
            break;
        }
    }
}

fn main(
    our: &Address,
    state: &mut State,
    db: &Sqlite,
    pending: &mut PendingLogs,
) -> anyhow::Result<()> {
    let message = await_message()?;
    match message {
        Message::Request { source, body, .. } => handle_request(our, &source, body, state, db),
        Message::Response {
            source,
            body,
            context,
            ..
        } => handle_response(our, &source, body, context, state, db, pending),
    }
}

fn handle_request(
    our: &Address,
    source: &Address,
    body: Vec<u8>,
    state: &mut State,
    db: &Sqlite,
) -> anyhow::Result<()> {
    let process = source.process.to_string();
    let pkg = source.package_id().to_string();
    // kiprintln!("process: {}\n{}", pkg, process);
    if pkg.as_str() == "terminal:sys" {
        handle_terminal_debug(&body, state)?;
    } else if process.as_str() == "http-server:distro:sys" {
        http_handlers::handle_frontend(our, &body, state, db)?;
    };

    Ok(())
}

fn handle_terminal_debug(body: &[u8], state: &mut State) -> anyhow::Result<()> {
    let bod = String::from_utf8(body.to_vec())?;
    // kiprintln!("terminal command: {}", bod);
    let command = bod.as_str();
    match command {
        "state" => {
            kiprintln!("hpn state\n{:#?}", state);
        }
        "reset" => {
            kiprintln!("resetting");
            state.reset();
        }
        _ => (),
    }
    Ok(())
}
fn handle_response(
    _our: &Address,
    source: &Address,
    body: Vec<u8>,
    context: Option<Vec<u8>>,
    state: &mut State,
    db: &Sqlite,
    pending: &mut PendingLogs,
) -> anyhow::Result<()> {
    let process = source.process.to_string();
    match process.as_str() {
        "timer:distro:sys" => {
            let is_checkpoint = context == Some(b"checkpoint".to_vec());
            chain::handle_timer(state, db, pending, is_checkpoint)?;
        }
        "eth:distro:sys" => {
            chain::handle_eth_message(state, db, pending, &body)?;
        }
        _ => (),
    };
    Ok(())
}
