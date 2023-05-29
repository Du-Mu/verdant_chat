use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use actix::*;
use actix_files::{Files, NamedFile};
use actix_identity::{Identity, IdentityMiddleware};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key, http::StatusCode, middleware::Logger, web, App, Error, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use actix_web_actors::ws;

mod api;
mod server;
mod session;

fn secret_key() -> Key {
    Key::generate()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // set up applications state
    // keep a count of the number of visitors
    let app_state = Arc::new(AtomicUsize::new(0));

    // start chat server actor
    let server = server::ChatServer::new(app_state.clone()).start();

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(app_state.clone()))
            .app_data(web::Data::new(server.clone()))
            .service(web::resource("/").route(web::get().to(api::index)))
            .service(web::resource("/login").route(web::post().to(api::login)))
            .service(Files::new("/static", "./static"))
            .wrap(Logger::default())
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key().clone(),
            ))
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
