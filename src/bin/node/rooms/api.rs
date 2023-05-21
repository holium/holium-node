use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CreateRoomPayload {
    rid: String,
    access: String,
    title: String,
    path: Option<String>,
}

#[derive(Debug)]
pub enum ActionType {
    SetProvider { ship: String },
    ResetProvider,
    CreateRoom(CreateRoomPayload),
    EditRoom,
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

pub fn parse_action_type(json: &str) -> anyhow::Result<ActionType> {
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
                println!("create-room action: {:?}", action_map.get("create-room"));
                let room: CreateRoomPayload = serde_json::from_value(
                    action_map
                        .get("create-room")
                        .ok_or(anyhow::anyhow!("missing create-room action"))?
                        .clone(),
                )
                .context("Failed to parse create-room action")?;
                ActionType::CreateRoom(room)
            } else if action_map.contains_key("edit-room") {
                ActionType::EditRoom
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
        ActionType::CreateRoom(data) => {
            println!(
                "Room created: {}, access: {:?}, {}, {}",
                data.rid,
                data.access,
                data.title,
                data.path.unwrap_or_default()
            );
        }
        ActionType::EditRoom {} => {
            println!("Room edited");
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
