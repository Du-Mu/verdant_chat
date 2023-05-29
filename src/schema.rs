// @generated automatically by Diesel CLI.

diesel::table! {
    messages (uuid) {
        uuid -> Char,
        time -> Timestamp,
        content -> Text,
        sender_id -> Char,
        room_id -> Integer,
    }
}

diesel::table! {
    rooms (id) {
        id -> Integer,
        rname -> Varchar,
    }
}

diesel::table! {
    users (uuid) {
        uuid -> Char,
        name -> Varchar,
        password -> Varchar,
        permission_id -> Integer,
    }
}

diesel::joinable!(messages -> rooms (room_id));
diesel::joinable!(messages -> users (sender_id));

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    rooms,
    users,
);
