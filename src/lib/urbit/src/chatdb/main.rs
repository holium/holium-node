use crate::error::Result;
use crate::Channel;
use crossbeam::channel::Receiver;

use bedrock_db::db::Db;
use serde_json::Value;

// todo: scry chat data and load into db
pub fn load(db: Db) -> Result<()> {
    Ok(())
}

pub struct ChatDb<'a> {
    pub channel: &'a mut Channel,
}

impl<'a> ChatDb<'a> {
    fn channel(&mut self) -> &mut Channel {
        self.channel
    }

    /// Technical Note: This method actually creates a new `Channel` with your Urbit Ship, and spawns a new unix thread
    /// locally that processes all messages on said channel. This is required due to borrowing mechanisms in Rust, however
    /// on the plus side this makes it potentially more performant by each subscription having it's own unix thread.
    fn subscribe(&mut self) -> Result<Receiver<AuthoredMessage>> {
        // Create sender/receiver
        let (s, r) = unbounded();
        // Creating a new Ship Interface Channel to pass into the new thread
        // to be used to communicate with the Urbit ship
        let mut new_channel = self.channel().ship_interface.create_channel()?;

        thread::spawn(move || {
            // Infinitely watch for new graph store updates
            let channel = &mut new_channel;
            channel.create_new_subscription("chat-db", "/db").ok();
            loop {
                channel.parse_event_messages();
                let res_updates = &mut channel.find_subscription("chat-db", "/db");
                if let Some(updates) = res_updates {
                    // Read all of the current SSE messages to find if any are for the resource
                    // we are looking for.
                    loop {
                        let pop_res = updates.pop_message();
                        // Acquire the message
                        if let Some(mess) = &pop_res {
                            // Parse it to json
                            if let Ok(json) = from_str::<Value>(mess) {
                                // TODO - Parse the json into a struct
                                println!("JSON: {}", json);
                            }
                        }
                        // If no messages left, stop
                        if let None = &pop_res {
                            break;
                        }
                    }
                }
                // Pause for half a second
                thread::sleep(Duration::new(0, 500000000));
            }
        });
        Ok(r)
    }

    pub fn initialize(
        pool: Pool<SqliteConnectionManager>,
        ship_interface: SafeShipInterface,
    ) -> Result<bool> {
        let mut channel = Channel::new(ship_interface);
        let mut chat_db = ChatDb { channel };
        chat_db.subscribe()?;
        Ok(chat_db)
    }
}
