// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpServer};
use rusqlite::Connection;
use std::{env, io::Result};

// Import modules
mod auth;
mod database;
mod services;
mod utils;

// This struct represents state
struct AppState {
    db: Connection,
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));

    // Generate session key in runtime so that restart invalidates older logins
    let secret_key = Key::generate();
    let db_location = env::var("db_url")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(String::from("urls.sqlite"));

    let port = env::var("port")
        .unwrap_or(String::from("4567"))
        .parse::<u16>()
        .expect("Supplied port is not an integer");

    let cache_control_header = env::var("cache_control_header")
        .ok()
        .filter(|s| !s.trim().is_empty());

    let api_url = {
        let mut get_api_url = env::var("api_url".replace("//", "/"))
        .ok()
        .unwrap_or_default();
        if get_api_url.ends_with("/") {
            get_api_url.pop();
        }
        get_api_url
    };

    // Actually start the server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .cookie_secure(false)
                    .build(),
            )
            // Maintain a single instance of database throughout
            .app_data(web::Data::new(AppState {
                db: database::open_db(db_location.clone()),
            }))
            .wrap(if let Some(header) = &cache_control_header {
                middleware::DefaultHeaders::new().add(("Cache-Control", header.to_owned()))
            } else {
                middleware::DefaultHeaders::new()
            })
            .service(services::link_handler)
            .service(web::scope(&api_url)
                .service(services::getall)
                .service(services::siteurl)
                .service(services::version)
                .service(services::add_link)
                .service(services::edit_link)
                .service(services::delete_link)
                .service(services::login)
                .service(services::gen_api_key)
                .service(services::logout)
                .service(Files::new("/", "./resources/").index_file("index.html"))
        )
            .default_service(actix_web::web::get().to(services::error404))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
