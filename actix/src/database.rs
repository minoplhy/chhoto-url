// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use rusqlite::{Connection, Error};
use serde::Serialize;

// Struct for encoding a DB row
#[derive(Serialize)]
pub struct DBRow {
    shortlink: String,
    longlink: String,
    hits: i64,
}

// Find a single URL
pub fn find_url(shortlink: &str, db: &Connection) -> Option<String> {
    let mut statement = db
        .prepare_cached("SELECT long_url FROM urls WHERE short_url = ?1")
        .expect("Error preparing SQL statement for find_url.");

    statement
        .query_row([shortlink], |row| row.get("long_url"))
        .ok()
}

// Get all URLs in DB
pub fn getall(db: &Connection) -> Vec<DBRow> {
    let mut statement = db
        .prepare_cached("SELECT * FROM urls")
        .expect("Error preparing SQL statement for getall.");

    let mut data = statement
        .query([])
        .expect("Error executing query for getall.");

    let mut links: Vec<DBRow> = Vec::new();
    while let Some(row) = data.next().expect("Error reading fetched rows.") {
        let row_struct = DBRow {
            shortlink: row
                .get("short_url")
                .expect("Error reading shortlink from row."),
            longlink: row
                .get("long_url")
                .expect("Error reading shortlink from row."),
            hits: row.get("hits").expect("Error reading shortlink from row."),
        };
        links.push(row_struct);
    }

    links
}

// Add a hit when site is visited
pub fn add_hit(shortlink: &str, db: &Connection) {
    db.execute(
        "UPDATE urls SET hits = hits + 1 WHERE short_url = ?1",
        [shortlink],
    )
    .expect("Error updating hit count.");
}

// Insert a new link
pub fn add_link(shortlink: String, longlink: String, db: &Connection) -> bool {
    db.execute(
        "INSERT INTO urls (long_url, short_url, hits) VALUES (?1, ?2, ?3)",
        (longlink, shortlink, 0),
    )
    .is_ok()
}

// Delete and existing link
pub fn delete_link(shortlink: String, db: &Connection) -> bool {
    if let Ok(delta) = db.execute("DELETE FROM urls WHERE short_url = ?1", [shortlink]) {
        delta > 0
    } else {
        false
    }
}

// Edit Existing Long link.
pub fn edit_link(shortlink: String, longlink: String, db: &Connection) -> bool {
    db.execute(
        "UPDATE urls SET long_url = ?1 WHERE short_url = ?2;", 
        [longlink, shortlink],
    )
    .is_ok()
}

pub fn add_api_key(api_key: String, db: &Connection) -> bool {
    db.execute(
        "INSERT OR REPLACE INTO api (id, api_key) VALUES (0, ?1);",
        [api_key]
    )
    .is_ok()
}

pub fn reset_api_key(db: &Connection) -> bool {
    db.execute(
        "DELETE FROM api WHERE id = 0;",
        []
    )
    .is_ok()
}

// get hashed api key from db, output as enum for later error processing
// etc -> no api key
pub fn get_api_key(db: &Connection) -> Result<String, Error> {
    db.query_row(
        "SELECT api_key FROM api WHERE id = 0",
        [],
        |row| row.get::<_, String>(0),
    )
}

// Open the DB, and create schema if missing
pub fn open_db(path: String) -> Connection {
    let db = Connection::open(path).expect("Unable to open database!");
    // Create table if it doesn't exist
    db.execute(
        "CREATE TABLE IF NOT EXISTS urls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            long_url TEXT NOT NULL,
            short_url TEXT NOT NULL,
            hits INTEGER NOT NULL
            )",
        [],
    )
    .expect("Unable to initialize empty database.");
    // create table if doesn't exist. For API key!
    db.execute("CREATE TABLE IF NOT EXISTS api (
        id INTEGER PRIMARY KEY CHECK (id = 0),
        api_key TEXT NOT NULL
        )",
         []
    )
    .expect("Unable to initialize empty database.");
    db
}
