// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_session::Session;
use actix_web::HttpRequest;
use rusqlite::Connection;
use std::time::SystemTime;

use crate::database;
use crate::config::Config;
use crate::utils::{hash_string, is_api_header};

// Validate a given password
pub fn validate(
    session: Session,
    config: &Config,
) -> bool {
    // If there's no password provided, just return true
    if config.password.is_none() {
        return true;
    }

    if let Ok(token) = session.get::<String>("chhoto-url-auth") {
        check(token)
    } else {
        false
    }
}

// Validate x-api-header to match the key in database
pub fn apikey_validate(httprequest: HttpRequest, db: &Connection) -> bool {
    let header = match httprequest.headers().get("x-api-key")
        .and_then(|h| h.to_str().ok()) {
            Some(key) if !key.is_empty() => key,
            _ => return false,
        };
    
    //  match with enum from db func, if no api key in row/others error -> return false
    match database::get_api_key(&db) {
        Ok(stored_key) => hash_string(&header.to_string()) == stored_key,
        Err(_) => {
            false
        }
    }}

pub fn authenticate(
    session: Session,
    httprequest: HttpRequest,
    db: &Connection,
    config: &Config, 
) -> bool {
    if !is_api_header(&httprequest) {
        if validate(session, &config) {
            return true;
        }
    }

    if apikey_validate(httprequest, db) {
        return true;
    }
    false
}

// Check a token cryptographically
fn check(token: Option<String>) -> bool {
    if let Some(token_body) = token {
        let token_parts: Vec<&str> = token_body.split(';').collect();
        if token_parts.len() < 2 {
            false
        } else {
            let token_text = token_parts[0];
            let token_time = token_parts[1].parse::<u64>().unwrap_or(0);
            let time_now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_secs();
            token_text == "chhoto-url-auth" && time_now < token_time + 1209600 // There are 1209600 seconds in 14 days
        }
    } else {
        false
    }
}

// Generate a new cryptographic token
pub fn gen_token() -> String {
    let token_text = String::from("chhoto-url-auth");
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards!")
        .as_secs();
    format!("{token_text};{time}")
}
