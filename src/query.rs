use crate::models;
use crate::models::{Mess, Pool, Room, User};
use actix_web::web;
use diesel::prelude::*;

pub fn query_user(user: &String, pool: web::Data<Pool>) -> Option<User> {
    use crate::schema::users::dsl::{name, users};
    let conn = &pool.get().expect("Fail to conect");
    let mut items = users.filter(name.eq(&user)).load::<User>(conn).unwrap();

    return items.pop();
}
pub fn query_user_from_id(user_id: &String, pool: web::Data<Pool>) -> Option<User> {
    use crate::schema::users::dsl::{users, uuid};
    let conn = &pool.get().expect("Fail to conect");
    let mut items = users.filter(uuid.eq(&user_id)).load::<User>(conn).unwrap();

    return items.pop();
}
pub fn delete_user(user: &String, pool: web::Data<Pool>) -> Result<usize, diesel::result::Error> {
    use crate::schema::users::dsl::{name, users};
    use crate::schema::messages::dsl::{messages, sender_id};
    let conn = &pool.get().expect("Fail to conect");
    if let Some(value) = query_user(user, pool){
        diesel::delete(messages.filter(sender_id.eq(&value.uuid))).execute(conn).expect("Fail to delete msg");
    }
    diesel::delete(users.filter(name.eq(&user))).execute(conn)
}
pub fn insert_user(user: &String, pass: &String,pool: web::Data<Pool>) -> Result<usize, diesel::result::Error>{
    use crate::schema::users::dsl::{users};
    let conn = &pool.get().expect("Fail to conect");
    let new_user = models::User::from_details(user, pass);

    diesel::insert_into(users).values(&new_user).execute(conn)
}
pub fn query_room(ro_name: &String, pool: web::Data<Pool>) -> Option<Room> {
    use crate::schema::rooms::dsl::{rname, rooms};
    let conn = &pool.get().expect("Fail to conect");
    let mut items = rooms.filter(rname.eq(&ro_name)).load::<Room>(conn).unwrap();

    return items.pop();
}
pub fn insert_room(
    ro_name: &String,
    pool: web::Data<Pool>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::schema::rooms::dsl::rooms;
    let conn = &pool.get().expect("Fail to conect");
    let new_room = models::Room {
        id: 0,
        rname: ro_name.to_owned(),
    };
    diesel::insert_into(rooms).values(&new_room).execute(conn)?;
    Ok(())
}
pub fn query_message(room_id_: i32, pool: web::Data<Pool>) -> Vec<Mess> {
    use crate::schema::messages::dsl::{messages, room_id};
    let conn = &pool.get().expect("Fail to conect");
    let items = messages
        .filter(room_id.eq(&room_id_))
        .load::<Mess>(conn)
        .unwrap();

    return items;
}
pub fn insert_message(
    msg: &String,
    room_id_: i32,
    sender_id_: &String,
    pool: web::Data<Pool>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::schema::messages::dsl::messages;
    let conn = &pool.get().expect("Fail to conect");
    let new_msg = models::Mess::from_details(sender_id_, msg, room_id_);
    diesel::insert_into(messages)
        .values(&new_msg)
        .execute(conn)?;
    Ok(())
}
