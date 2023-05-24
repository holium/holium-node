// use serde::Deserialize;

use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

use super::{
    api::{CreateRoomPayload, EditRoomPayload, SourceShip},
    state::{ProviderState, SessionState},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub rid: String,
    pub provider: String,
    pub creator: String,
    pub access: String,
    pub title: String,
    pub present: Vec<String>,
    pub whitelist: Vec<String>,
    pub capacity: u32,
    pub path: Option<String>,
}

impl Room {
    pub fn new(create_payload: CreateRoomPayload) -> Room {
        Room {
            rid: create_payload.rid,
            provider: String::new(),
            creator: String::new(),
            access: create_payload.access,
            title: create_payload.title,
            present: Vec::new(),
            whitelist: Vec::new(),
            capacity: 8,
            path: create_payload.path,
            // db_pool: db_pool,
        }
    }
}

#[derive(Debug)]
pub struct RoomsState {
    pub provider: ProviderState,
    pub session: SessionState,
}

impl RoomsState {
    pub fn new() -> RoomsState {
        let provider = ProviderState::new();
        let session = SessionState::new();

        RoomsState { provider, session }
    }

    pub fn initialize(&mut self, identity: String) {
        self.provider.initialize(identity.clone());
        self.session.initialize(identity.clone());
    }

    pub fn create_room(
        &mut self,
        source_ship: SourceShip,
        create_payload: CreateRoomPayload,
    ) -> Room {
        let mut room = Room::new(create_payload);
        room.provider = self.provider.identity.clone();
        room.creator = source_ship.clone();
        self.session.rooms.insert(room.rid.clone(), room.clone());

        room
    }

    pub fn edit_room(&mut self, source_ship: SourceShip, edit_payload: EditRoomPayload) {
        let mut room = self
            .session
            .rooms
            .get_mut(&edit_payload.rid)
            .expect("Room not found");
        if (room.creator != source_ship) || (self.provider.identity != source_ship) {
            return;
        }
        room.access = edit_payload.access.clone();
        room.title = edit_payload.title.clone();
    }

    pub fn delete_room(&mut self, source_ship: SourceShip, rid: String) {
        let room = self.session.rooms.get_mut(&rid).expect("Room not found");
        if (room.creator != source_ship) || (self.provider.identity != source_ship) {
            return;
        }
        self.session.rooms.remove(&rid);
    }

    // pub fn enter_room(&mut self, source_ship: SourceShip, rid: String) {
    //     let mut room = self.session.rooms.get_mut(&rid).expect("Room not found");
    //     if room.access == "public" {
    //         room.present.push(source_ship.clone());
    //     } else if room.access == "invite" {
    //         if room.whitelist.contains(&source_ship) {
    //             room.present.push(source_ship.clone());
    //         }
    //     }
    // }
}

lazy_static! {
    pub static ref ROOMS_STATE: Arc<Mutex<RoomsState>> = Arc::new(Mutex::new(RoomsState::new()));
}
