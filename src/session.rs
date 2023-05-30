use crate::query;
use crate::server;
use crate::models::Pool;
use actix::prelude::*;
use actix_web::web;
use actix_web_actors::ws;
use std::time::{Duration, Instant};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsChatSession {
    /// unique session id
    pub id: String,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,

    /// joined room
    pub room: String,

    /// peer name
    pub name: Option<String>,

    /// Chat server
    pub addr: Addr<server::ChatServer>,

    pub db_pool: web::Data<Pool>,
}

impl WsChatSession {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(server::Disconnect { id: act.id.clone() });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        if let Some(db_room) = query::query_room(&"main".to_string(), self.db_pool.clone()) {
            log::info!("{} exit", db_room.rname);
        } else {
            query::insert_room(&"main".to_string(), self.db_pool.clone()).expect("Fail to insert to room");
        }

        let addr = ctx.address();
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
                id: self.id.clone(),
            })
            .into_actor(self)
            .then(|res, act, ctx| fut::ready(()))
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.addr.do_send(server::Disconnect {
            id: self.id.clone(),
        });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => {
                            // Send ListRooms message to chat server and wait for
                            // response
                            println!("List rooms");
                            self.addr
                                .send(server::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(room);
                                            }
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx)
                            // .wait(ctx) pauses all events in context,
                            // so actor wont receive any new messages until it get list
                            // of rooms back
                        }
                        "/join" => {
                            if v.len() == 2 {
                                self.room = v[1].to_owned();
                                if let Some(db_room) = query::query_room(&self.room, self.db_pool.clone())
                                {
                                    log::info!("{} exist", db_room.rname);
                                } else {
                                    query::insert_room(&self.room, self.db_pool.clone())
                                        .expect("Fail to insert value");
                                }
                                self.addr.do_send(server::Join {
                                    id: self.id.clone(),
                                    name: self.room.clone(),
                                });

                                ctx.text("joined");
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                self.name = Some(v[1].to_owned());
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }
                        "/history" => {
                            if let Some(now_room) = query::query_room(&self.room, self.db_pool.clone()) {
                                for i in query::query_message(now_room.id, self.db_pool.clone()) {
                                    if let Some(value) = query::query_user_from_id(&i.sender_id, self.db_pool.clone()) {
                                        ctx.text(value.name+":"+&i.content)
                                    }
                                    
                                }
                            }
                        }
                        "/rm" => {
                            if v.len() == 2 {
                                if let Some(src_peo) = query::query_user_from_id(&self.id, self.db_pool.clone()) {
                                    if src_peo.permission_id == 1 {
                                       query::delete_user(&v[1].to_string(), self.db_pool.clone()).expect("Faile to delete user");
                                    }
                                }
                            } else {
                                ctx.text("!!! name is required");
                            }

                        }
                        _ => ctx.text(format!("!!! unknown command: {m:?}")),
                    }
                } else {
                    let msg = if let Some(ref name) = self.name {
                        format!("{name}: {m}")
                    } else {
                        m.to_owned()
                    };
                    if let Some(now_room) = query::query_room(&self.room, self.db_pool.clone()) {
                        query::insert_message(&msg, now_room.id, &self.id, self.db_pool.clone())
                            .expect("Fail to insert to msg");
                    }
                    // send message to chat server
                    self.addr.do_send(server::ClientMessage {
                        id: self.id.clone(),
                        msg,
                        room: self.room.clone(),
                    })
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
