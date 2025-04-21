use chrono::TimeZone;
use hyperware_process_lib::homepage::add_to_homepage;
use hyperware_process_lib::http::server::{HttpBindingConfig, HttpServer};
use hyperware_process_lib::logging::{info, init_logging, Level};
use hyperware_process_lib::sqlite::Sqlite;
use hyperware_process_lib::{await_message, call_init, Address, Message};

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
    let db = db::load_db(&our).unwrap();

    let mut pending = chain::start_fetch(&mut state, &db);
    loop {
        if let Err(e) = main(&our, &mut state, &db, &mut pending) {
            // print_to_terminal(1, "fatal error {e}");
            info!("something wrong at main\n{:#?}", e);
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
        Message::Request { source, body, .. } => {
            handle_request(our, &source, body, state, db, pending)
        }
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
    pending: &mut PendingLogs,
) -> anyhow::Result<()> {
    let process = source.process.to_string();
    let pkg = source.package_id().to_string();
    // kiprintln!("process: {}\n{}", pkg, process);
    if pkg.as_str() == "terminal:sys" {
        handle_terminal_debug(our, &body, state, db, pending)?;
    } else if process.as_str() == "http-server:distro:sys" {
        http_handlers::handle_frontend(our, &body, state, db)?;
    } else {
        let request = serde_json::from_slice::<ClientRequest>(&body)?;
        info!("{:#?}", request);
        http_handlers::handle_client_request(request, db)?;
    }

    Ok(())
}

fn handle_terminal_debug(
    our: &Address,
    body: &[u8],
    state: &mut State,
    db: &Sqlite,
    pending: &mut PendingLogs,
) -> anyhow::Result<()> {
    let bod = String::from_utf8(body.to_vec())?;
    // kiprintln!("terminal command: {}", bod);
    let command = bod.as_str();
    match command {
        "state" => {
            info!("hpn state\n{:#?}", state);
            info!("block: {:#?}", state.last_checkpoint_block);
            let datetime = chrono::Utc.timestamp_opt(state.logging_started as i64, 0);
            info!("started: {:#?}", datetime);
        }
        "db" => {
            let db_up = db::check_schema(db);
            info!("hpn db\n{:#?}", db_up);
        }
        "reset" => {
            info!("block: {:#?}", state.last_checkpoint_block);
            let nstate = State::new();
            *state = nstate;
            info!("block: {:#?}", state.last_checkpoint_block);
            info!("resetting db");
            db::wipe_db(our)?;
            db::load_db(our)?;
            let npending = chain::start_fetch(state, db);
            *pending = npending;
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
