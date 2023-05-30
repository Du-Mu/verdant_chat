use std::{
    sync::{
        atomic::AtomicUsize,
        Arc,
    },
};

use actix::*;
use actix_files::{Files};
use actix_identity::{IdentityMiddleware};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    middleware::Logger, web, App,
    HttpServer,
};
//use actix::*;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

#[macro_use]
extern crate diesel;
mod models;
mod schema;

mod api;
mod query;
mod server;
mod session;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    dotenv::dotenv().ok();
    std::env::set_var(
        "RUST_LOG",
        "simple-auth-server=debug,actix_web=info,actix_server=info",
    );
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // create db connection pool
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    let pool: models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // set up applications state
    // keep a count of the number of visitors
    let app_state = Arc::new(AtomicUsize::new(0));

    // start chat server actor
    let server = server::ChatServer::new(app_state.clone()).start();

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::from(app_state.clone()))
            .app_data(web::Data::new(server.clone()))
            .service(web::resource("/").route(web::get().to(api::index)))
            .service(web::resource("/login").route(web::post().to(api::login)))
            .service(web::resource("/rigister").route(web::get().to(api::rigister)))
            .service(web::resource("/rigister_post").route(web::post().to(api::rigister_post)))
            .service(Files::new("/static", "./static"))
            .service(web::resource("/chatroom").to(api::chatroom))
            .route("/count", web::get().to(api::get_count))
            .route("/ws", web::get().to(api::chat_route))
            .wrap(Logger::default())
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                api::secret_key().clone(),
            ))
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
