use core::str;
use std::collections::HashMap;

use hyperware_process_lib::http::client::send_request_await_response;
use hyperware_process_lib::http::Method;
use hyperware_process_lib::logging::{error, info, init_logging, Level};
use hyperware_process_lib::{await_message, call_init, Address, Message, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use url::Url;

use hyperware_process_lib::vfs::{create_drive, open_dir, open_file};
wit_bindgen::generate!({
    path: "target/wit",
    world: "hpn-sortugdev-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});
mod structs;
use structs::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
enum ProviderRequest {
    API(ApiKeyHandling),
    MCP(MCPRequest),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
enum ApiKeyHandling {
    Set(User),
    Del(String),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
struct MCPRequest {
    provider_name: String,
    arguments: HashMap<String, Value>,
}
fn call_json_api(url: &str, headers: Option<HashMap<String, String>>) -> anyhow::Result<Vec<u8>> {
    info!("calling {}", url);
    let mut hs = HashMap::new();
    hs.insert("Content-type".to_string(), "application/json".to_string());
    if let Some(heds) = headers {
        hs.extend(heds);
    }
    let url = Url::parse(&url)?;
    let res = send_request_await_response(Method::GET, url, Some(hs), 60, vec![]);
    match res {
        Err(e) => {
            let error_body = serde_json::to_vec(&e.to_string())?;
            Ok(error_body)
        }
        Ok(r) => Ok(r.body().to_vec()),
    }
}

fn catpic_list(our: &Address) -> Value {
    let drive_path = create_drive(our.package_id(), "pkg", None).unwrap();
    let drive_path2 = drive_path[1..].to_string();
    let folder_path = format!("{}/pics/", drive_path2);
    let dir = open_dir(&format!("{}/pics", drive_path), false, None).unwrap();
    let files = dir
        .read()
        .unwrap()
        .iter()
        .map(|f| f.path.replace(&folder_path, ""))
        .collect::<Vec<String>>();
    json!(files)
}
fn get_catpic(our: &Address, file_name: &str) -> anyhow::Result<Vec<u8>> {
    let drive_path = create_drive(our.package_id(), "pkg", None).unwrap();
    info!("{}", drive_path);
    let file_path = format!("{}/pics/{}", drive_path, file_name);
    let file = open_file(&file_path, false, None)?;
    let file_bytes = file.read()?;
    Ok(file_bytes)
}
fn handle_api_request(req: ApiKeyHandling, state: &mut State) -> anyhow::Result<()> {
    match req {
        ApiKeyHandling::Set(u) => {
            let wallet = u.wallet.clone();
            state.in_keys.insert(wallet, u);
        }
        ApiKeyHandling::Del(s) => {
            state.in_keys.remove(&s);
        }
    }
    Ok(())
}
fn handle_mcp_request(our: &Address, req: MCPRequest, state: &mut State) -> anyhow::Result<()> {
    info!("{:#?}", req);
    match req.provider_name.as_str() {
        "WeatherAPI.com" => {
            let api_key = state
                .out_keys
                .get("WEATHER_API_KEY")
                .ok_or(anyhow::anyhow!("no outgoing API KEY"))?;
            let query_value = req
                .arguments
                .get("query")
                .ok_or(anyhow::anyhow!("bad arguments sent"))?;
            let query = query_value
                .as_str()
                .ok_or(anyhow::anyhow!("bad query argument sent"))?;
            let url = format!(
                "https://api.weatherapi.com/v1/current.json?key={}&q={}",
                api_key, query
            );
            let res_body = call_json_api(&url, None)?;
            Response::new().body(res_body).send()?;
        }
        "Finnhub API" => {
            let api_key = state
                .out_keys
                .get("FINNHUB_API_KEY")
                .ok_or(anyhow::anyhow!("no outgoing API KEY"))?;
            let symbol_value = req
                .arguments
                .get("symbol")
                .ok_or(anyhow::anyhow!("bad arguments sent"))?;
            let symbol = symbol_value
                .as_str()
                .ok_or(anyhow::anyhow!("bad symbol argument sent"))?;
            let url = format!(
                "https://finnhub.io/api/v1/quote?symbol={}&token={}",
                symbol, api_key
            );
            let res_body = call_json_api(&url, None)?;
            Response::new().body(res_body).send()?;
        }
        "dune" => {
            let api_key = state
                .out_keys
                .get("DUNE_API_KEY")
                .ok_or(anyhow::anyhow!("no outgoing API KEY"))?;
            let contract = req
                .arguments
                .get("contract")
                .ok_or(anyhow::anyhow!("bad arguments sent"))?
                .as_str()
                .ok_or(anyhow::anyhow!("bad arguments sent"))?;
            let chain_idv = req
                .arguments
                .get("chainId")
                .ok_or(anyhow::anyhow!("bad arguments sent"))?;
            let chain_id = match chain_idv {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.to_string(),
                _ => return Err(anyhow::anyhow!("bad arguments sent")),
            };
            let url = format!(
                "https://api.dune.com/api/echo/beta/tokens/evm/{}?chain_ids={}",
                contract, chain_id
            );
            let mut headers = HashMap::new();
            headers.insert("X-Dune-Api-Key".to_string(), api_key.to_string());
            let res_body = call_json_api(&url, Some(headers))?;
            Response::new().body(res_body).send()?;
        }
        "Catpics" => {
            let query_value = req
                .arguments
                .get("query")
                .ok_or(anyhow::anyhow!("bad arguments sent"))?;
            match query_value.to_owned() {
                Value::String(s) => {
                    if s.as_str() == "list" {
                        let catpic_list = catpic_list(our);
                        Response::new()
                            .body(serde_json::to_vec(&catpic_list)?)
                            .send()?;
                    } else {
                        return Err(anyhow::anyhow!("bad arguments sent"));
                    }
                }
                Value::Object(obj) => {
                    let file_name = obj
                        .get("file")
                        .ok_or(anyhow::anyhow!("bad arguments sent"))?
                        .as_str()
                        .ok_or(anyhow::anyhow!("bad arguments sent"))?;
                    let catpic = get_catpic(our, file_name);
                    match catpic {
                        Ok(bytes) => {
                            Response::new().body(bytes).send()?;
                        }
                        Err(e) => return Err(e),
                    }
                }
                _ => return Err(anyhow::anyhow!("bad arguments sent")),
            }
            info!("{:#?}", req.arguments);
            ()
        }
        _ => (),
    };
    Ok(())
}
fn handle_message(our: &Address, message: &Message, state: &mut State) -> anyhow::Result<()> {
    if !message.is_request() {
        return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
    }

    let body = message.body();
    let source = message.source();
    let source_node = source.node();
    let pkg = source.package_id().to_string();
    if source_node == our.node() && pkg.as_str() == "terminal:sys" {
        return handle_terminal(our, body, state);
    }
    // let jsonstring = String::from_utf8(body.to_vec())?;
    let req = serde_json::from_slice::<ProviderRequest>(body)?;
    match req {
        ProviderRequest::API(r) => handle_api_request(r, state),
        ProviderRequest::MCP(mcp) => handle_mcp_request(our, mcp, state),
    }
}

call_init!(init);
fn init(our: Address) {
    init_logging(Level::DEBUG, Level::INFO, None, None, None).unwrap();
    info!("begin");

    let mut state = State::load();
    loop {
        match await_message() {
            Err(send_error) => error!("got SendError: {send_error}"),
            Ok(ref message) => match handle_message(&our, message, &mut state) {
                Ok(_) => {}
                Err(e) => error!("got error while handling message: {e:?}"),
            },
        }
    }
}

fn handle_terminal(_our: &Address, body: &[u8], state: &mut State) -> anyhow::Result<()> {
    let bod = String::from_utf8(body.to_vec())?;
    let mut words = bod.split_whitespace();
    let command = words.next().ok_or(anyhow::anyhow!("bad command"))?;
    match command {
        "state" => {
            info!("provider state\n{:#?}", state);
        }
        "add-key" => {
            let name = words.next().ok_or(anyhow::anyhow!("bad command"))?;
            let key = words.next().ok_or(anyhow::anyhow!("bad command"))?;
            state.out_keys.insert(name.to_string(), key.to_string());
        }
        "del-key" => {
            let name = words.next().ok_or(anyhow::anyhow!("bad command"))?;
            state.out_keys.remove(name);
        }
        "reset" => {
            let nstate = State::new();
            *state = nstate;
            state.save();
        }
        _ => (),
    }
    Ok(())
}
