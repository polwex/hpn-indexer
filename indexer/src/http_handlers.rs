use std::collections::HashMap;

use crate::{db as dbm, structs::*};
use hyperware_process_lib::http::server::{send_response, HttpServerRequest};
use hyperware_process_lib::http::{Method, StatusCode};
use hyperware_process_lib::sqlite::Sqlite;
use hyperware_process_lib::{kiprintln, last_blob, Address};
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
            let prefix = format!("{}:{}/api", our.process(), our.package_id());
            let path = req.bound_path(Some(&prefix));
            kiprintln!("request path: {}", path);
            let met = req.method()?;
            match met {
                Method::GET => {
                    match handle_get(our, path, req.query_params(), state, db) {
                        Ok(_) => (),
                        Err(e) => {
                            kiprintln!("error handling get request\n{:#?}", e);
                            send_response(StatusCode::INTERNAL_SERVER_ERROR, None, vec![]);
                        }
                    };
                }
                Method::POST => match handle_post(db) {
                    Ok(_) => (),
                    Err(e) => {
                        kiprintln!("error handling post request\n{:#?}", e);
                        send_response(StatusCode::SERVICE_UNAVAILABLE, None, vec![]);
                    }
                },
                _ => {
                    send_response(StatusCode::METHOD_NOT_ALLOWED, None, vec![]);
                }
            }
        }
        _ => (),
    };
    Ok(())
}
fn handle_post(db: &Sqlite) -> anyhow::Result<()> {
    let blob = last_blob().ok_or(anyhow::anyhow!("no blob"))?;
    // let json = std::str::from_utf8(blob.bytes());
    // kiprintln!("json\n:{:#?}", json);
    let body = serde_json::from_slice::<HttpPostRequest>(blob.bytes())?;
    handle_mcp(db, body)?;
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
        "/mcp" => {
            let mut headers = HashMap::new();
            headers.insert("Content-type".to_string(), "text/event-stream".to_string());
            headers.insert("Connection".to_string(), "keep-alive".to_string());
            headers.insert("Cache-Control".to_string(), "no-cache".to_string());
            let message = "data: {message: oh hai!}\r\n\r\n".as_bytes().to_vec();
            send_response(StatusCode::OK, Some(headers), message);
        }
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

fn handle_mcp(db: &Sqlite, req: HttpPostRequest) -> anyhow::Result<()> {
    kiprintln!("mcp request\n{:#?}", req);
    let res_body = match req {
        HttpPostRequest::SearchRegistry(query) => {
            let data = dbm::search_provider(db, query)?;
            data
        }
    };
    kiprintln!("mcp response\n{:#?}", res_body);
    send_json_response(StatusCode::OK, &json!(res_body))?;
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
