use anyhow::{anyhow, Error, Result};
// use hex::ToHex;
use hyperware_process_lib::{
    kiprintln,
    sqlite::{self, Sqlite},
    Address,
};
use serde_json::Value;
// use serde_json::Value::Null;
// use std::collections::HashMap;
use std::collections::HashMap;

use crate::helpers::make_json_timestamp;

pub fn open_db(our: &Address) -> Result<sqlite::Sqlite, Error> {
    let p = our.package_id();
    // let rmv = sqlite::remove_db(p.clone(), "hpn-explorer", None);
    let db = sqlite::open(p, "hpn-explorer", None);
    db
}

pub fn check_schema(db: &Sqlite) -> anyhow::Result<()> {
    let required = ["providers"];
    let mut found = required
        .iter()
        .map(|&s| (s, false))
        .collect::<std::collections::HashMap<_, _>>();

    let statement = "SELECT name from sqlite_master WHERE type='table';".to_string();
    let data = db.read(statement, vec![])?;
    kiprintln!("sqlite read {:?}", data);
    let values: Vec<Value> = data
        .iter()
        .filter_map(|map| map.get("name"))
        .cloned()
        .collect();

    kiprintln!("sql tables:{:?}", values);
    for val in values {
        if let Value::String(s) = val {
            if let Some(entry) = found.get_mut(s.as_str()) {
                *entry = true;
            }
        }
    }
    let good = found.values().all(|&b| b);
    if good {
        return Ok(());
    } else {
        return write_db_schema(db);
    }
}

pub fn write_db_schema(db: &Sqlite) -> anyhow::Result<()> {
    let tx_id = db.begin_tx()?;
    let s0 = "CREATE TABLE categories(name TEXT PRIMARY KEY, hash TEXT NOT NULL);".to_string();
    let s1 = r#"
        CREATE TABLE providers(
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          hash TEXT NOT NULL UNIQUE,
          name TEXT NOT NULL UNIQUE,
          provider_name TEXT,
          site TEXT,
          description TEXT,
          category TEXT NOT NULL,
          created INTEGER,
          FOREIGN KEY (category) REFERENCES categories(name)
        );"#
    .to_string();
    let s2 = r#"
       CREATE INDEX idx_providers_category
       ON providers (id, category);
    "#
    .to_string();
    db.write(s0, vec![], Some(tx_id))?;
    db.write(s1, vec![], Some(tx_id))?;
    db.write(s2, vec![], Some(tx_id))?;
    return db.commit_tx(tx_id);
}
// reads
// writes
pub fn insert_category(db: &Sqlite, child_hash: String, name: String) -> Result<(), Error> {
    let s1 = r#"
        INSERT OR IGNORE INTO categories(name, hash) 
        VALUES (?1, ?2);
        "#
    .to_string();
    let p1 = vec![
        serde_json::Value::String(name),
        serde_json::Value::String(child_hash),
    ];
    db.write(s1, p1, None)
}
pub fn insert_provider(
    db: &Sqlite,
    parent_hash: &str,
    child_hash: String,
    name: String,
) -> Result<(), Error> {
    // kiprintln!("inserting provider\n{:#?}", provider);
    let category = get_category(db, parent_hash.to_string())?;
    let category = category.get(0).ok_or(anyhow!("no category"))?;
    let category = category.get("name").ok_or(anyhow!("no category name"))?;
    let category = category.to_owned();
    let s1 = r#"
        INSERT OR IGNORE INTO providers(hash, name, category, created) 
        VALUES (?1, ?2, ?3, ?4);
        "#
    .to_string();
    let now = make_json_timestamp();
    let p1 = vec![
        serde_json::Value::String(child_hash),
        serde_json::Value::String(name),
        category,
        serde_json::Value::Number(now),
    ];
    db.write(s1, p1, None)
}
pub fn insert_provider_facts(
    db: &Sqlite,
    key: String,
    value: String,
    hash: String,
) -> Result<(), Error> {
    let s1 = format!(
        r#"
        UPDATE providers SET
        '{}' = ?1
        WHERE hash = ?2
        "#,
        key
    );
    // kiprintln!("{}-> {}", key, value);
    let p1 = vec![
        serde_json::Value::String(value),
        serde_json::Value::String(hash),
    ];
    db.write(s1, p1, None)
}
// reads
pub fn get_all(db: &Sqlite) -> Result<Vec<HashMap<String, Value>>> {
    let s = "SELECT * FROM providers".to_string();
    let data = db.read(s, vec![])?;
    Ok(data)
}
pub fn get_category(db: &Sqlite, hash: String) -> Result<Vec<HashMap<String, Value>>> {
    let s = "SELECT * FROM categories WHERE hash = ?1".to_string();
    let h = serde_json::Value::String(hash);
    let data = db.read(s, vec![h])?;
    Ok(data)
}
pub fn get_by_category(db: &Sqlite, category: String) -> Result<Vec<HashMap<String, Value>>> {
    let s = "SELECT * FROM providers WHERE category= ?1".to_string();
    let h = serde_json::Value::String(category);
    let data = db.read(s, vec![h])?;
    Ok(data)
}
pub fn search_provider(db: &Sqlite, query: String) -> Result<Vec<HashMap<String, Value>>> {
    let param = format!(r#"'%{}%'"#, query);
    let s = format!(
        r#"
        SELECT * FROM providers
        WHERE (category LIKE {0} COLLATE NOCASE)
        OR (name LIKE {0} COLLATE NOCASE)
        OR (provider_name LIKE {0} COLLATE NOCASE)
        OR (site LIKE {0} COLLATE NOCASE)
        OR (description LIKE {0} COLLATE NOCASE)
        "#,
        param
    );
    let data = db.read(s, vec![])?;
    Ok(data)
}

// getters

// pub fn get_casts(db: &Sqlite, fid: u64) -> Result<Vec<CastRes>> {
//     let db = open_db(our)?;
//     let statement = r#"
//     SELECT text,
//      hash,
//      author,
//      embeds,
//      mentions,
//      mentions_positions,
//      casts.created,
//      fid,
//      username,
//      name,
//      bio,
//      pfp,
//      url
//      FROM casts
//      LEFT JOIN user_data ON casts.author = user_data.fid
//      WHERE author = ?
//      "#
//     .to_string();
//     let params = vec![fid.into()];
//     let data = db.read(statement, params);
//     println!("reading casts {:?} {:?}", fid, data);
//     let data = data.unwrap();
//     let res = process_casts(our, data)?;
//     Ok(res)
// }
// pub fn process_casts(db: &Sqlite, data: Vec<HashMap<String, Value>>) -> Result<Vec<CastRes>> {
//     let mut res = vec![];
//     for dbres in data {
//         let cast = CastT::from_sqlite(&dbres);
//         println!("cast {:?}", cast);
//         if let Err(_) = cast {
//             continue;
//         }
//         let author = match Profile::from_sqlite(&dbres) {
//             Err(err) => {
//                 println!("failed parsing profile {:?}", err);
//                 None
//             }
//             Ok(p) => Some(p),
//         };
//         println!("parsed profile {:?}", author);
//         let cast = cast.unwrap();
//         let (rts, likes, replies) = get_engagement(our, &cast, true);
//         let reply_count = replies.len();
//         let cr = CastRes {
//             author,
//             cast,
//             rts,
//             likes,
//             replies,
//             reply_count,
//         };
//         res.push(cr);
//     }
//     Ok(res)
// }
