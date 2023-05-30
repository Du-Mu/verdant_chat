// models.rs
use super::schema::*;
use diesel::{r2d2::ConnectionManager, Insertable, MysqlConnection, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// type alias to use in multiple places
pub type Pool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    pub uuid: String,
    pub name: String,
    pub password: String,
    pub permission_id: i32,
}
impl User {
    pub fn from_details<S: Into<String>, T: Into<String>>(user: S, pass: T) -> Self {
        User {
            name: user.into(),
            password: pass.into(),
            uuid: Uuid::new_v4().clone().to_string(),
            permission_id: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "rooms"]
pub struct Room {
    pub id: i32,
    pub rname: String,
}
impl Room {
    pub fn from_details<T: Into<String>>(rname: T) -> Self {
        Room {
            rname: rname.into(),
            id: 0,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "messages"]
pub struct Mess {
    pub uuid: String,
    pub time: chrono::NaiveDateTime,
    pub content: String,
    pub sender_id: String,
    pub room_id: i32,
}
impl Mess {
    pub fn from_details<S: Into<String>, T: Into<String>>(sender: S, cont: T, room: i32) -> Self {
        Mess {
            uuid: Uuid::new_v4().clone().to_string(),
            time: chrono::Local::now().naive_local() + chrono::Duration::hours(24),
            content: cont.into(),
            sender_id: sender.into(),
            room_id: room.into()
        }
    }
}
