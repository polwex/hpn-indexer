use core::str;
use std::collections::HashMap;

use hyperware_process_lib::http::client::send_request_await_response;
use hyperware_process_lib::http::Method;
use hyperware_process_lib::logging::{error, info, init_logging, Level};
use hyperware_process_lib::{await_message, call_init, kiprintln, Address, Message, Response};
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

type ApiKeys = HashMap<String, String>;
const TEMP_KEY: &str = "henlo";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct User {
    wallet: String,
    tx_hash: String,
}
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
const WEATHER_API_KEY: &str = "1ef55da5e2844b1995b115659251104";
const FINNHUB_API_KEY: &str = "cvsfqupr01qhup0qhvr0cvsfqupr01qhup0qhvrg";
const DUNE_API_KEY: &str = "fi0V4dqNdMhKQorm1rhJ9XIZ84gDpkWW";
fn call_json_api(url: &str) -> anyhow::Result<Vec<u8>> {
    kiprintln!("calling {}", url);
    let mut headers = HashMap::new();
    headers.insert("Content-type".to_string(), "application/json".to_string());
    let url = Url::parse(&url)?;
    let res = send_request_await_response(Method::GET, url, Some(headers), 5000, vec![])?;
    // kiprintln!("res body\n{:#?}", String::from_utf8(res_body.clone())?);
    Ok(res.body().to_vec())
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
fn handle_api_request(req: ApiKeyHandling, keys: &mut ApiKeys) -> anyhow::Result<()> {
    match req {
        ApiKeyHandling::Set(_u) => {
            info!("tbd");
        }
        ApiKeyHandling::Del(s) => {
            keys.remove(&s);
        }
    }
    Ok(())
}
fn handle_mcp_request(
    our: &Address,
    req: MCPRequest,
    _api_keys: &mut ApiKeys,
) -> anyhow::Result<()> {
    kiprintln!("{:#?}", req);
    match req.provider_name.as_str() {
        "WeatherAPI.com" => {
            let query_value = req
                .arguments
                .get("query")
                .ok_or(anyhow::anyhow!("bad arguments sent"))?;
            let query = query_value
                .as_str()
                .ok_or(anyhow::anyhow!("bad query argument sent"))?;
            let url = format!(
                "https://api.weatherapi.com/v1/current.json?key={}&q={}",
                WEATHER_API_KEY, query
            );
            let res_body = call_json_api(&url)?;
            Response::new().body(res_body).send()?;
        }
        "Finnhub API" => {
            let symbol_value = req
                .arguments
                .get("symbol")
                .ok_or(anyhow::anyhow!("bad arguments sent"))?;
            let symbol = symbol_value
                .as_str()
                .ok_or(anyhow::anyhow!("bad symbol argument sent"))?;
            let url = format!(
                "https://finnhub.io/api/v1/quote?symbol={}&token={}",
                symbol, FINNHUB_API_KEY
            );
            let res_body = call_json_api(&url)?;
            Response::new().body(res_body).send()?;
        }
        "dune" => {
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
            headers.insert("Content-type".to_string(), "application/json".to_string());
            headers.insert("X-Dune-Api-Key".to_string(), DUNE_API_KEY.to_string());
            let url = Url::parse(&url)?;
            let res = send_request_await_response(Method::GET, url, Some(headers), 5000, vec![])?;
            let res_body = res.body().to_vec();
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
            kiprintln!("{:#?}", req.arguments);
            ()
        }
        _ => (),
    };
    Ok(())
}
fn handle_message(our: &Address, message: &Message, api_keys: &mut ApiKeys) -> anyhow::Result<()> {
    if !message.is_request() {
        return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
    }

    let body = message.body();
    let jsonstring = String::from_utf8(body.to_vec())?;
    kiprintln!("{}", jsonstring);
    let req = serde_json::from_slice::<ProviderRequest>(body)?;
    match req {
        ProviderRequest::API(r) => handle_api_request(r, api_keys),
        ProviderRequest::MCP(mcp) => handle_mcp_request(our, mcp, api_keys),
    }
}

call_init!(init);
fn init(our: Address) {
    init_logging(Level::DEBUG, Level::INFO, None, None, None).unwrap();
    info!("begin");
    catpic_list(&our);

    let mut api_keys = HashMap::new();
    api_keys.insert("tmp".to_string(), TEMP_KEY.to_string());
    loop {
        match await_message() {
            Err(send_error) => error!("got SendError: {send_error}"),
            Ok(ref message) => match handle_message(&our, message, &mut api_keys) {
                Ok(_) => {}
                Err(e) => error!("got error while handling message: {e:?}"),
            },
        }
    }
}
