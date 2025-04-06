use std::collections::HashMap;

use alloy_sol_types::SolEvent;
use hyperware_process_lib::eth::Filter;
use hyperware_process_lib::sqlite::Sqlite;
use hyperware_process_lib::{eth, hypermap, print_to_terminal, println, timer};

use crate::db as dbm;
use crate::helpers::decode_datakey;
use crate::structs::*;
use alloy_primitives::keccak256;

const MAX_PENDING_ATTEMPTS: u8 = 3;
// const SUBSCRIPTION_TIMEOUT: u64 = 60;
pub fn make_filters(state: &State) -> (eth::Filter, eth::Filter) {
    // print_to_terminal(
    //     2,
    //     &format!("hypermap.address {}", &hypermap.address().to_string()),
    // );
    let address = state.hypermap.address().to_owned();
    let mint_filter = eth::Filter::new()
        .address(address.clone())
        .from_block(state.last_checkpoint_block)
        .to_block(eth::BlockNumberOrTag::Latest)
        .event(hypermap::contract::Mint::SIGNATURE);
    // TODO expand this when it's time
    let notes_filter = eth::Filter::new()
        .address(address)
        .from_block(state.last_checkpoint_block)
        .to_block(eth::BlockNumberOrTag::Latest)
        .event(hypermap::contract::Note::SIGNATURE)
        .topic3(vec![
            keccak256("~site"),
            keccak256("~description"),
            keccak256("~provider-name"),
        ]);
    (mint_filter, notes_filter)
}
pub fn start_fetch(state: &mut State, db: &Sqlite) -> PendingLogs {
    let (mints, notes) = make_filters(&state);
    state
        .hypermap
        .provider
        .subscribe_loop(1, mints.clone(), 0, 0);
    state
        .hypermap
        .provider
        .subscribe_loop(2, notes.clone(), 0, 0);

    let mut pending_logs: PendingLogs = Vec::new();
    // set a timer tick so any pending logs will be processed
    timer::set_timer(DELAY_MS, None);

    // set a timer tick for checkpointing
    timer::set_timer(CHECKPOINT_MS, Some(b"checkpoint".to_vec()));
    //
    fetch_and_process_logs(state, db, &mut pending_logs, &mints);
    fetch_and_process_logs(state, db, &mut pending_logs, &notes);
    pending_logs
}

fn fetch_and_process_logs(
    state: &mut State,
    db: &Sqlite,
    pending: &mut PendingLogs,
    filter: &Filter,
) {
    loop {
        match state.hypermap.provider.get_logs(filter) {
            Ok(logs) => {
                print_to_terminal(2, &format!("log len: {}", logs.len()));
                for log in logs {
                    if let Err(e) = handle_log(state, db, pending, &log, 0) {
                        print_to_terminal(1, &format!("log-handling error! {e:?}"));
                    }
                }
                return;
            }
            Err(e) => {
                println!("got eth error while fetching logs: {e:?}, trying again in 5s...");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        }
    }
}

// provider notes
// ~provider-name
// ~description
// ~site
// all under hpn=testing-beta.os and categories

pub fn handle_log(
    state: &mut State,
    db: &Sqlite,
    pending: &mut PendingLogs,
    log: &eth::Log,
    attempt: u8,
) -> anyhow::Result<()> {
    let processed = match log.topics()[0] {
        hypermap::contract::Mint::SIGNATURE_HASH => {
            let decoded = hypermap::contract::Mint::decode_log_data(log.data(), true).unwrap();
            let parent_hash = decoded.parenthash.to_string();
            let child_hash = decoded.childhash.to_string();
            let label = String::from_utf8(decoded.label.to_vec())?;

            add_mint(state, db, &parent_hash, child_hash, label)
        }
        hypermap::contract::Note::SIGNATURE_HASH => {
            let decoded = hypermap::contract::Note::decode_log_data(log.data(), true).unwrap();

            let parent_hash = decoded.parenthash.to_string();
            let note_label = String::from_utf8(decoded.label.to_vec())?;

            add_note(state, db, &parent_hash, note_label, decoded.data)
        }
        // hypermap::contract::Fact::SIGNATURE_HASH => {
        //     // let decoded = hypermap::contract::Fact::decode_log_data(log.data(), true).unwrap();
        //     // let parent_hash = decoded.parenthash.to_string();
        //     // let fact_label = String::from_utf8(decoded.label.to_vec())?;

        //     // self.add_fact(&parent_hash, fact_label, decoded.data)?;
        // }
        _ => Ok(()),
    };

    if let Some(block_number) = log.block_number {
        state.last_checkpoint_block = block_number;
    }

    match processed {
        Ok(_) => (),
        Err(_e) => {
            pending.push((log.to_owned(), attempt + 1));
        }
    };

    Ok(())
}

pub fn add_mint(
    state: &mut State,
    db: &Sqlite,
    parent_hash: &str,
    child_hash: String,
    name: String,
) -> anyhow::Result<()> {
    // kiprintln!("adding mint\n{}\n{}\n{}", name, parent_hash, child_hash);
    if name == "hpn-testing-beta" {
        state.root_hash = Some(child_hash);
        // 0xdeeac81ae11b64e7cab86d089c306e5d223552a630f02633ce170d2786ff1bbd
        // 0xb1c62e68d4a8ae7bc27f784515f2da8534b876415af7c523d94689f563a880fd
        return Ok(());
    }

    let root = state.root_hash.clone().unwrap_or("oops".to_string());
    let parent = parent_hash.to_string();
    if parent == root {
        state.categories.insert(child_hash.clone(), name.clone());
        dbm::insert_category(db, child_hash.clone(), name.clone())?;
        return Ok(());
    };
    let _db_insert = dbm::insert_provider(db, parent_hash, child_hash.clone(), name.clone());
    if let Some(category) = state.categories.get(parent_hash) {
        let provider = Provider {
            category: category.to_owned(),
            name: name.clone(),
            hash: child_hash.clone(),
            facts: HashMap::new(),
        };
        state.providers.insert(child_hash.clone(), provider.clone());
        return Ok(());
    };

    // kiprintln!(
    //     "failed processing mint \n{:#?}\n{}\n{}",
    //     name,
    //     parent_hash,
    //     child_hash
    // );
    Err(anyhow::anyhow!("pending"))
}
pub fn add_note(
    state: &mut State,
    db: &Sqlite,
    parent_hash: &str,
    note_label: String,
    data: eth::Bytes,
) -> anyhow::Result<()> {
    // kiprintln!("adding note\n{}\n{}", note_label, parent_hash);
    // remove the ~
    let key = note_label
        .chars()
        .skip(1)
        .collect::<String>()
        .replace("-", "_");
    let decoded = decode_datakey(&data.to_string())?;
    dbm::insert_provider_facts(db, key, decoded.clone(), parent_hash.to_string())?;
    if let Some(provider) = state.providers.get_mut(parent_hash) {
        let facts = provider.facts.get_mut(&note_label);
        match facts {
            None => {
                let datav = vec![decoded];
                provider.facts.insert(note_label, datav);
            }
            Some(f) => {
                f.push(decoded);
            }
        }
        return Ok(());
    };
    Err(anyhow::anyhow!("no parent"))
}

fn handle_pending(state: &mut State, db: &Sqlite, pending: &mut PendingLogs) {
    let mut newpending: PendingLogs = Vec::new();
    for (log, attempt) in pending.drain(..) {
        if attempt >= MAX_PENDING_ATTEMPTS {
            continue;
        }
        let _ = handle_log(state, db, &mut newpending, &log, attempt);
    }
    pending.extend(newpending);
}
pub fn handle_eth_message(
    state: &mut State,
    db: &Sqlite,
    pending: &mut PendingLogs,
    body: &[u8],
) -> anyhow::Result<()> {
    match serde_json::from_slice::<eth::EthSubResult>(body) {
        Ok(Ok(eth::EthSub { result, .. })) => {
            if let Ok(eth::SubscriptionResult::Log(log)) =
                serde_json::from_value::<eth::SubscriptionResult>(result)
            {
                if let Err(e) = handle_log(state, db, pending, &log, 0) {
                    print_to_terminal(1, &format!(" log-handling error! {e:?}"));
                }
            }
        }
        Ok(Err(e)) => {
            println!("got eth subscription error ({e:?}), resubscribing");
            let address = state.hypermap.address().to_owned();
            if e.id == 1 {
                let filter = eth::Filter::new()
                    .address(address)
                    .from_block(state.last_checkpoint_block)
                    .to_block(eth::BlockNumberOrTag::Latest)
                    .event(hypermap::contract::Mint::SIGNATURE);
                state.hypermap.provider.subscribe_loop(1, filter, 2, 0);
            } else if e.id == 2 {
                let filter = eth::Filter::new()
                    .address(address)
                    .from_block(state.last_checkpoint_block)
                    .to_block(eth::BlockNumberOrTag::Latest)
                    .event(hypermap::contract::Note::SIGNATURE)
                    .topic3(vec![
                        keccak256("~site"),
                        keccak256("~description"),
                        keccak256("~provider-name"),
                    ]);
                state.hypermap.provider.subscribe_loop(2, filter, 2, 0);
            }
        }
        _ => {}
    }

    Ok(())
}
pub fn handle_timer(
    state: &mut State,
    db: &Sqlite,
    pending: &mut PendingLogs,
    is_checkpoint: bool,
) -> anyhow::Result<()> {
    let block_number = state.hypermap.provider.get_block_number();
    if let Ok(block_number) = block_number {
        print_to_terminal(2, &format!("new block: {}", block_number));
        state.last_checkpoint_block = block_number;
        if is_checkpoint {
            state.save();
            timer::set_timer(CHECKPOINT_MS, Some(b"checkpoint".to_vec()));
        }
    }
    handle_pending(state, db, pending);

    if !pending.is_empty() {
        timer::set_timer(DELAY_MS, None);
    }
    Ok(())
}
