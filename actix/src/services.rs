// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::NamedFile;
use actix_session::Session;
use actix_web::{
    delete, get, http::StatusCode, post, put, web::{self, Redirect}, Either, HttpRequest, HttpResponse, Responder
};
use std::env;

use crate::auth;
use crate::database;
use crate::utils;
use crate::AppState;

// Store the version number
const VERSION: &str = env!("CARGO_PKG_VERSION");

// Define the routes

// Add new links
#[post("/api/new")]
pub async fn add_link(
    req: String, 
    data: web::Data<AppState>, 
    session: Session, 
    httprequest: HttpRequest) 
    -> HttpResponse {
    
    let config = &data.config;
    if config.public_mode || auth::validate(session) || auth::apikey_validate(httprequest, &data.db) {
        let (is_ok, output_url) = utils::add_link(req, &data.db);
        if is_ok {
            HttpResponse::Created().body(output_url)
        } else {
            HttpResponse::Conflict().body(output_url)
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

// Return all active links
#[get("/api/all")]
pub async fn getall(
    data: web::Data<AppState>, 
    session: Session, 
    httprequest: HttpRequest
) -> HttpResponse {
    
    let config = &data.config;
    if auth::validate(session) || auth::apikey_validate(httprequest, &data.db) {
        HttpResponse::Ok().body(utils::getall(&data.db))
    } else {
        let body = if config.public_mode {
            "Using public mode."
        } else {
            "Not logged in!"
        };
        HttpResponse::Unauthorized().body(body)
    }
}

// Get the site URL
#[get("/api/siteurl")]
pub async fn siteurl(
    data: web::Data<AppState>
) -> HttpResponse {
    HttpResponse::Ok().body(data.config.site_url.clone())
}

// Get the version number
#[get("/api/version")]
pub async fn version() -> HttpResponse {
    HttpResponse::Ok().body(VERSION)
}

// 404 error page
pub async fn error404() -> impl Responder {
    NamedFile::open_async("./resources/static/404.html")
        .await
        .customize()
        .with_status(StatusCode::NOT_FOUND)
}

// Handle a given shortlink
#[get("/{shortlink}")]
pub async fn link_handler(
    shortlink: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let shortlink_str = shortlink.to_string();
    if let Some(longlink) = utils::get_longurl(shortlink_str, &data.db) {
        database::add_hit(shortlink.as_str(), &data.db);
        if &data.config.redirect_method == "TEMPORARY" {
            Either::Left(Redirect::to(longlink))
        } else {
            // Defaults to permanent redirection
            Either::Left(Redirect::to(longlink).permanent())
        }
    } else {
        Either::Right(
            NamedFile::open_async("./resources/static/404.html")
                .await
                .customize()
                .with_status(StatusCode::NOT_FOUND),
        )
    }
}

// Handle login
#[post("/api/login")]
pub async fn login(
    req: String,
    session: Session,
    data: web::Data<AppState>,
) -> HttpResponse {
    if let Some(ref password) = data.config.password {
        if password != &req {
            eprintln!("Failed login attempt!");
            return HttpResponse::Unauthorized().body("Wrong password!");
        }
    }
    // Return Ok if no password was set on the server side
    session
        .insert("chhoto-url-auth", auth::gen_token())
        .expect("Error inserting auth token.");
    HttpResponse::Ok().body("Correct password!")
}

// Create API Key
#[post("/api/key")]
pub async fn gen_api_key(session: Session, httprequest: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    if auth::validate(session) || auth::apikey_validate(httprequest, &data.db) {
        let (is_ok, output) = utils::gen_api_key(&data.db);
        if is_ok {
            HttpResponse::Ok().body(output)
        } else {
            HttpResponse::Conflict().body("Generate Api Key Error!")
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

// Handle logout
#[delete("/api/logout")]
pub async fn logout(session: Session) -> HttpResponse {
    if session.remove("chhoto-url-auth").is_some() {
        HttpResponse::Ok().body("Logged out!")
    } else {
        HttpResponse::Unauthorized().body("You don't seem to be logged in.")
    }
}

// Edit link
#[put("/api/edit/{shortlink}")]
pub async fn edit_link(
    req: String, 
    shortlink: web::Path<String>,
    data: web::Data<AppState>, 
    session: Session,
    httprequest: HttpRequest,
) -> HttpResponse {
    if auth::validate(session) || auth::apikey_validate(httprequest, &data.db) {
        let (is_ok, output) = utils::edit_link(req, shortlink.to_string(), &data.db);
        if is_ok {
            HttpResponse::Created().body(output)
        } else {
            HttpResponse::Conflict().body(output)
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

// Delete a given shortlink
#[delete("/api/del/{shortlink}")]
pub async fn delete_link(
    shortlink: web::Path<String>,
    data: web::Data<AppState>,
    session: Session,
    httprequest: HttpRequest,
) -> HttpResponse {
    if auth::validate(session) || auth::apikey_validate(httprequest, &data.db) {
        if utils::delete_link(shortlink.to_string(), &data.db) {
            HttpResponse::Ok().body(format!("Deleted {shortlink}"))
        } else {
            HttpResponse::NotFound().body("Not found!")
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}