use anyhow::Context;
use serde::Deserialize;

use crate::room::ROOMS_STATE;

#[derive(Deserialize, Debug)]
pub struct CreateRoomPayload {
    pub rid: String,
    pub access: String,
    pub title: String,
    pub path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct EditRoomPayload {
    pub rid: String,
    pub access: String,
    pub title: String,
}

pub type SourceShip = String;

#[derive(Debug)]
pub enum ActionType {
    SetProvider { ship: String },
    ResetProvider,
    CreateRoom(SourceShip, CreateRoomPayload),
    EditRoom(SourceShip, EditRoomPayload),
    DeleteRoom,
    EnterRoom,
    LeaveRoom,
    Invite,
    Kick,
    SendChat,
}

// #[derive(Debug)]
// enum ReactionType {
//     RoomEntered,
//     RoomCreated,
//     RoomLeft,
//     RoomUpdated,
//     RoomDeleted,
//     ProviderChanged,
//     Invited,
//     Kick,
//     ChatReceived,
// }

pub fn parse_action_type(json_tuple: (String, String)) -> anyhow::Result<ActionType> {
    let source_ship = json_tuple.0;
    let json = json_tuple.1.as_str();
    let value: serde_json::Value =
        serde_json::from_str(json).context("Failed to parse json in parse_action_type")?;

    let action = match value.as_object() {
        Some(action_map) => {
            if action_map.contains_key("set-provider") {
                let ship = action_map
                    .get("set-provider")
                    .and_then(|v| v.as_str())
                    .ok_or(anyhow::anyhow!("invalid set-provider action"))?
                    .to_string();
                ActionType::SetProvider { ship }
            } else if action_map.contains_key("reset-provider") {
                ActionType::ResetProvider
            } else if action_map.contains_key("create-room") {
                // println!("create-room action: {:?}", action_map.get("create-room"));
                let room: CreateRoomPayload = serde_json::from_value(
                    action_map
                        .get("create-room")
                        .ok_or(anyhow::anyhow!("missing create-room action"))?
                        .clone(),
                )
                .context("Failed to parse create-room action")?;
                ActionType::CreateRoom(source_ship, room)
            } else if action_map.contains_key("edit-room") {
                let room_edit: EditRoomPayload = serde_json::from_value(
                    action_map
                        .get("edit-room")
                        .ok_or(anyhow::anyhow!("missing create-room action"))?
                        .clone(),
                )
                .context("Failed to parse edit-room action")?;
                ActionType::EditRoom(source_ship, room_edit)
            } else if action_map.contains_key("delete-room") {
                ActionType::DeleteRoom
            } else if action_map.contains_key("enter-room") {
                ActionType::EnterRoom
            } else if action_map.contains_key("leave-room") {
                ActionType::LeaveRoom
            } else if action_map.contains_key("invite") {
                ActionType::Invite
            } else if action_map.contains_key("kick") {
                ActionType::Kick
            } else if action_map.contains_key("send-chat") {
                ActionType::SendChat
            } else {
                return Err(anyhow::anyhow!("invalid action type"));
            }
        }
        None => return Err(anyhow::anyhow!("invalid action type")),
    };

    Ok(action)
}

pub async fn handle_action(action: ActionType) -> Result<(), warp::Rejection> {
    match action {
        ActionType::SetProvider { ship } => {
            // Handle room entered event...
            println!("Set provider to: {:?}", ship);
        }
        ActionType::ResetProvider {} => {
            // Handle room created event...
            println!("Reset to default provider");
        }
        ActionType::CreateRoom(source_ship, data) => {
            println!("{} is creating a room called {}", source_ship, data.title);
            {
                let mut rooms_state = ROOMS_STATE.lock().unwrap();
                rooms_state.create_room(source_ship, data);
            }
        }
        ActionType::EditRoom(source_ship, data) => {
            println!("{} is editing a room {}", source_ship, data.rid);
            {
                let mut rooms_state = ROOMS_STATE.lock().unwrap();
                rooms_state.edit_room(source_ship, data);
            }
        }
        ActionType::DeleteRoom {} => {
            println!("Room deleted");
        }
        ActionType::EnterRoom {} => {
            println!("Entered room");
        }
        ActionType::LeaveRoom {} => {
            println!("Left room");
        }
        ActionType::Invite {} => {
            println!("Invited");
        }
        ActionType::Kick {} => {
            println!("Kicked");
        }
        ActionType::SendChat {} => {
            println!("Sent chat");
        }
    }

    Ok(())
}

pub async fn handle_get_session() -> Result<impl warp::Reply, warp::Rejection> {
    let session = {
        let rooms_state = ROOMS_STATE.lock().unwrap();
        rooms_state.session.clone()
    };
    Ok(warp::reply::json(&session))
}
