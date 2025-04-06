use std::collections::HashMap;

use crate::{db as dbm, structs::*};
use hyperware_process_lib::http::server::{send_response, HttpServerRequest};
use hyperware_process_lib::http::{Method, StatusCode};
use hyperware_process_lib::sqlite::Sqlite;
use hyperware_process_lib::{kiprintln, Address};
use serde_json::json;

pub fn handle_frontend(
    our: &Address,
    body: &[u8],
    state: &mut State,
    db: &Sqlite,
) -> anyhow::Result<()> {
    let server_request: HttpServerRequest = serde_json::from_slice(body)?;
    match server_request {
        HttpServerRequest::Http(req) => {
            let met = req.method()?;
            match met {
                Method::GET => {
                    let prefix = format!("{}:{}/api", our.process(), our.package_id());
                    let path = req.bound_path(Some(&prefix));
                    handle_get(our, path, req.query_params(), state, db)?;
                }
                _ => (),
            }
        }
        _ => (),
    };
    Ok(())
}

fn handle_get(
    our: &Address,
    path: &str,
    params: &HashMap<String, String>,
    state: &mut State,
    db: &Sqlite,
) -> anyhow::Result<()> {
    match path {
        "/state" => {
            send_json_response(StatusCode::OK, &json!(state.providers))?;
        }
        "/all" => {
            let data = dbm::get_all(db)?;
            send_json_response(StatusCode::OK, &json!(data))?;
        }
        "/cat" => {
            let query = params.get("cat").ok_or(anyhow::anyhow!("no category"))?;
            let data = dbm::get_by_category(db, query.to_string())?;
            send_json_response(StatusCode::OK, &json!(data))?;
        }
        "/search" => {
            let query = params.get("q").ok_or(anyhow::anyhow!("no query"))?;
            let data = dbm::search_provider(db, query.to_string())?;
            send_json_response(StatusCode::OK, &json!(data))?;
        }
        _ => {
            send_json_response(StatusCode::NOT_FOUND, &json!(false))?;
        }
    };
    Ok(())
}

fn send_json_response<T: serde::Serialize>(status: StatusCode, data: &T) -> anyhow::Result<()> {
    let json_data = serde_json::to_vec(data)?;
    send_response(
        status,
        Some(HashMap::from([(
            String::from("Content-Type"),
            String::from("application/json"),
        )])),
        json_data,
    );
    Ok(())
}
