use core::str;
use hyperware_process_lib::{get_state, println, set_state};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// add from terminal
// ex:
// m our@provider:hpn:sortugdev.os 'add-key WEATHER_API_KEY 1ef55da5e2844b1995b115659251104'
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    pub out_keys: HashMap<String, String>,
    pub in_keys: HashMap<String, User>,
}
impl State {
    pub fn new() -> Self {
        let state = State {
            out_keys: HashMap::new(),
            in_keys: HashMap::new(),
        };
        state
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
    }

    /// Reset by removing the checkpoint and reloading fresh state
    // pub fn reset(&self) {
    //     clear_state();
    // }

    /// Saves a checkpoint, serializes to the current block
    pub fn save(&mut self) {
        match serde_json::to_vec(self) {
            Ok(state_bytes) => set_state(&state_bytes),
            Err(e) => println!("failed to serialize state: {e:?}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub wallet: String,
    pub tx_hash: String,
    pub api_key: String,
}
