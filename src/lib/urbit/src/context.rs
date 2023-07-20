use std::sync::Arc;

use crate::api::Ship;
use crate::db::Db;
use serde_json::Value as JsonValue;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

#[derive(Debug)]
pub struct NodeContext {
    //
    //  gives access to the underlying SQLite database pool. all database
    //  interaction should happen thru this interface. if the capability does not exist on
    //  this interface, add it. the purpose is to abstract from the underlying database
    //  vendor (in case we need to switch providers (e.g. postgresql, et. al.) in the future)
    //
    pub db: Db,

    ///
    // need Arc::Mutex since cookie expiration retries need to modify internal Ship
    //  state (e.g. update session_auth data in Ship instance). Arc::Mutex allows
    //  mutable references which is necessary for modifying internal Ship instance state.
    //
    pub ship: Ship,

    //
    //  the unbounded sender/receiver pair defined here are to facilitate the flow
    //  of events coming in from the Urbit ship (EventSource) to the connected
    //  devices over websocket
    //

    //  when new events come in from the bound ship, the data is cast (as json) and sent
    //  over this end of the channel. the ship_event_receiver (defined further down) will
    //  receive these new messages in a message loop
    pub sender: UnboundedSender<JsonValue>,

    //
    //  receiver - the output end of an unbounded channel that receives
    //    data sent by the UnboundedSender of the channnel.
    //
    //  calling the recv() method on the UnboundedReceiver requires a mutable
    //  reference. Mutex supports acquiring a mutable reference across threads
    //
    pub receiver: Arc<Mutex<UnboundedReceiver<JsonValue>>>,
}

// by "wrapping" the NodeContext in an Arc, we ensure that cloning
//  does not duplicate (allocate new) memory; and rather increases and
//  internal reference count (ARC - atomic reference count) variable "pointing"
//  to the same underlying memory
pub type CallContext = Arc<NodeContext>;

impl NodeContext {
    pub fn to_call_context(ctx: NodeContext) -> CallContext {
        Arc::new(ctx)
    }
}
