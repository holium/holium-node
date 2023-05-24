use super::api::CreateRoomPayload;
use super::room::Room;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

pub type Rooms = HashMap<String, Room>;

#[derive(Debug, Serialize, Clone)]
pub struct SessionState {
    pub provider: String,
    pub current: Option<String>,
    pub rooms: Rooms,
}

impl SessionState {
    pub fn new() -> Self {
        SessionState {
            provider: String::new(),
            current: None,
            rooms: HashMap::new(),
        }
    }
    pub fn initialize(&mut self, provider: String) {
        self.provider = provider;
    }

    pub fn get_rooms(&self) -> &Rooms {
        &self.rooms
    }

    pub fn get_room(&self, rid: &str) -> Option<&Room> {
        self.rooms.get(rid)
    }

    // pub fn set_provider(&mut self, provider: String) {
    //     self.provider = provider;
    // }
    pub fn create_room(&mut self, create_payload: CreateRoomPayload) {
        let room = Room::new(create_payload);
        self.rooms.insert(room.rid.clone(), room);
    }
    // pub fn delete_room(&mut self, rid: String) {
    //     self.rooms.remove(&rid);
    // }
    // pub fn edit_room(&mut self, room: Room) {}
    // pub fn enter_room(&mut self, room: Room) {}
    // pub fn leave_room(&mut self, room: Room) {}
    // pub fn invite(&mut self, room: Room) {
    //     self.rooms.insert(room.rid.clone(), room);
    // }
    // pub fn kick(&mut self, room: Room) {
    //     self.rooms.insert(room.rid.clone(), room);
    // }
    // pub fn send_chat(&mut self, room: Room) {
    //     self.rooms.insert(room.rid.clone(), room);
    // }
}

#[derive(Debug)]
pub struct ProviderState {
    pub identity: String,
    pub rooms: Rooms,
    pub online: bool,
    pub banned: HashSet<String>,
}

impl ProviderState {
    pub fn new() -> Self {
        ProviderState {
            identity: String::new(),
            rooms: HashMap::new(),
            online: true,
            banned: HashSet::new(),
        }
    }

    // Add an initialize function
    pub fn initialize(&mut self, identity: String) {
        self.identity = identity;
    }

    pub fn get_identity(&mut self) -> String {
        self.identity.clone()
    }

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
