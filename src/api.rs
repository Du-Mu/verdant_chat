use actix_files::NamedFile;
use actix_session::Session;
use actix_web::{
    dev, error, http::StatusCode, middleware::ErrorHandlerResponse, web, Error, HttpResponse,
    Responder, Result,
};
use serde::Deserialize;

pub async fn index(session: Session) -> NamedFile {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

#[derive(Debug, Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

pub async fn login(
    params: web::Form<LoginInfo>,
    session: Session,
) -> Result<impl Responder, Error> {
    let params = params.into_inner();
    let username = &params.username;
    
    session.insert("islogin", true).unwrap();
    session.insert("name", username).unwrap();

    log::info!("[{username}]:logging");

    Ok(web::Redirect::to("/").using_status_code(StatusCode::FOUND))
}
