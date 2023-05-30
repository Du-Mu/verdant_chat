use crate::{
    models::Pool,
    query,
};
use actix::Addr;
use actix_files::NamedFile;
use actix_http::HttpMessage;
use actix_identity::Identity;
use actix_web::{
    cookie::Key, http::StatusCode, web, Error,
    HttpRequest, HttpResponse, Responder, Result,
};
use actix_web_actors::ws;
use serde::Deserialize;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

use sha256::digest;

use crate::server;
use crate::session;

pub async fn index() -> NamedFile {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

pub async fn rigister() -> NamedFile {
    NamedFile::open_async("./static/rigister.html")
        .await
        .unwrap()
}

pub fn secret_key() -> Key {
    Key::generate()
}

pub async fn chatroom() -> impl Responder {
    NamedFile::open_async("./static/chatroom.html")
        .await
        .unwrap()
}

/// Entry point for our websocket route
pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
    user: Option<Identity>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    if let Some(user) = user {
        format!("Welcome! {}", user.id().unwrap());
        ws::start(
            session::WsChatSession {
                id: user.id().unwrap(),
                hb: Instant::now(),
                room: "main".to_owned(),
                name: None,
                addr: srv.get_ref().clone(),
                db_pool: pool,
            },
            &req,
            stream,
        )
    } else {
        Ok(HttpResponse::new(StatusCode::NON_AUTHORITATIVE_INFORMATION))
    }
}

/// Displays state
pub async fn get_count(count: web::Data<AtomicUsize>) -> impl Responder {
    let current_count = count.load(Ordering::SeqCst);
    format!("Visitors: {current_count}")
}

#[derive(Debug, Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

pub async fn login(
    pool: web::Data<Pool>,
    params: web::Form<LoginInfo>,
    request: HttpRequest,
) -> Result<impl Responder, Error> {
    let params = params.into_inner();
    let user_na = &params.username;
    let pass_wo = &params.password;
    log::info!("[{user_na}]:logging");
    if let Some(value) = query::query_user(user_na, pool) {
        if value.password.as_str() == digest(pass_wo.as_str()) {
            log::info!("[{user_na}]:login sucess");
            Identity::login(&request.extensions_mut(), value.uuid.into()).unwrap();
        } else {
            println!("{}", value.password.as_str());
            println!("{}", digest(pass_wo.as_str()));
        }
    } else {
        log::info!("[{user_na}]:login failed");
    }
    Ok(web::Redirect::to("/chatroom").using_status_code(StatusCode::FOUND))
}

#[derive(Debug, Deserialize)]
pub struct RigisterInfo {
    username: String,
    password: String,
}

pub async fn rigister_post(
    pool: web::Data<Pool>,
    params: web::Form<RigisterInfo>,
) -> Result<impl Responder, Error> {
    let params = params.into_inner();
    let user_na = &params.username;
    let pass_wo = &params.password;
    log::info!("[{user_na}]:rigistering");
    if let Some(value) = query::query_user(user_na, pool.clone()) {
        log::info!("[{}]:have been used", value.name);
        return Ok(web::Redirect::to("/rigister").using_status_code(StatusCode::FOUND));
    } else {
        query::insert_user(user_na, &digest(pass_wo.to_owned()), pool).expect("Fail to insert user");
    }
    Ok(web::Redirect::to("/login").using_status_code(StatusCode::FOUND))
}
