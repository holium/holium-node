use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
};
use warp::Filter;

pub mod api;
pub mod room;

use room::{Rid, Room};

pub type Rooms = HashMap<Rid, Room>;

#[derive(Debug)]
pub struct SessionState {
    provider: String,
    current: Option<Rid>,
    rooms: Rooms,
}

impl SessionState {
    fn new(provider: String) -> Self {
        SessionState {
            provider,
            current: None,
            rooms: HashMap::new(),
        }
    }
    fn change_provider(&mut self, provider: String) {
        self.provider = provider;
    }
    fn update_current(&mut self, rid: Rid) {
        self.current = Some(rid);
    }
    fn add_room(&mut self, room: Room) {
        self.rooms.insert(room.rid.clone(), room);
    }
    fn remove_room(&mut self, rid: Rid) {
        self.rooms.remove(&rid);
    }
    fn update_room(&mut self, room: Room) {
        self.rooms.insert(room.rid.clone(), room);
    }
}

#[derive(Debug)]
pub struct ProviderState {
    rooms: Rooms,
    online: bool,
    banned: HashSet<String>,
}

impl ProviderState {
    pub fn set_online(&mut self, online: bool) {
        self.online = online;
    }

    pub fn ban(&mut self, ship: String) {
        self.banned.insert(ship);
    }
    pub fn unban(&mut self, ship: String) {
        self.banned.remove(&ship);
    }
}
