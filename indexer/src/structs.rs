use hyperware_process_lib::{eth, get_state, hypermap, println, set_state};
use serde::{Deserialize, Serialize};
use serde_json::Value;
type Namehash = String;
use std::{
    collections::HashMap,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use hyperware::process::standard::clear_state;
wit_bindgen::generate!({
    path: "target/wit",
    world: "hpn-sortugdev-dot-os-v0",
    generate_unused_types: true,
    additional_derives: [serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

// pub use hyperware::process::hpn::{
//     HpnMessage, Request as HpnRequest, Response as HpnResponse, SendRequest,
// };

const HYPERMAP_ADDRESS: &'static str = hypermap::HYPERMAP_ADDRESS;

const CHAIN_ID: u64 = hypermap::HYPERMAP_CHAIN_ID; // base

const HYPERMAP_FIRST_BLOCK: u64 = hypermap::HYPERMAP_FIRST_BLOCK; // base

pub const DELAY_MS: u64 = 5_000; // 5s
pub const CHECKPOINT_MS: u64 = 300_000; // 5 minutes

// calls from the MCP shim
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum HttpPostRequest {
    SearchRegistry(String),
    #[serde(rename_all = "camelCase")]
    CallProvider {
        provider_id: String,
        provider_name: String,
        arguments: HashMap<String, Value>,
    },
}
#[derive(Clone, Debug, Deserialize, Serialize)]
enum DataKey {
    /// facts are immutable
    Fact(eth::Bytes),
    /// notes are mutable: we store all versions of the note, most recent last
    /// if indexing full history, this will be the note's full history --
    /// it is also possible to receive a snapshot and not have updates from before that.
    Note(Vec<eth::Bytes>),
}

type Name = String;
pub type PendingLogs = Vec<(eth::Log, u8)>;
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Provider {
    /// everything that comes before a name, from root, with dots separating and a leading dot
    pub category: String,
    /// the name of the node -- a string.
    pub name: Name,
    pub hash: String,
    pub facts: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct State {
    /// the chain id we are indexing
    pub chain_id: u64,
    /// what contract this state pertains to
    pub contract_address: eth::Address,
    /// namehash to human readable name
    pub hypermap: hypermap::Hypermap,
    // root hash of the hpn node
    pub root_hash: Option<Namehash>,
    pub categories: HashMap<String, String>,
    pub providers: HashMap<String, Provider>,
    pub names: HashMap<String, String>,
    /// last saved checkpoint block
    pub last_checkpoint_block: u64,
    pub logging_started: u64,
}

impl State {
    pub fn new() -> Self {
        let hypermap = hypermap::Hypermap::default(60);

        let new_state = Self {
            chain_id: CHAIN_ID,
            contract_address: eth::Address::from_str(HYPERMAP_ADDRESS).unwrap(),
            hypermap,
            root_hash: None,
            categories: HashMap::new(),
            providers: HashMap::new(),
            names: HashMap::from([(String::new(), hypermap::HYPERMAP_ROOT_HASH.to_string())]),
            last_checkpoint_block: HYPERMAP_FIRST_BLOCK,
            logging_started: get_now(),
        };
        new_state
    }

    pub fn load() -> Self {
        match get_state() {
            None => Self::new(),
            Some(state_bytes) => match serde_json::from_slice(&state_bytes) {
                Ok(state) => state,
                Err(e) => {
                    println!("failed to deserialize saved state: {e:?}");
                    Self::new()
                }
            },
        }
        // TEMP to start from scratch
        // Self::new()
    }

    /// Reset by removing the checkpoint and reloading fresh state
    pub fn reset(&self) {
        clear_state();
    }

    /// Saves a checkpoint, serializes to the current block
    pub fn save(&mut self) {
        match serde_json::to_vec(self) {
            Ok(state_bytes) => set_state(&state_bytes),
            Err(e) => println!("failed to serialize state: {e:?}"),
        }
    }
    // pub fn save(&mut self, block: u64) {
    //     self.last_checkpoint_block = block;
    //     match rmp_serde::to_vec(self) {
    //         Ok(state_bytes) => set_state(&state_bytes),
    //         Err(e) => println!("failed to serialize state: {e:?}"),
    //     }
    // }
}

// HTTP requests from frontend
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum FrontendRequest {
    Get,
    Other,
}

pub fn get_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
