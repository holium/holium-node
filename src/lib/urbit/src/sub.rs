use crate::context::CallContext;
use anyhow::{bail, Result};

pub async fn start(ctx: CallContext) -> Result<()> {
    let result = ctx.ship.start_listener(ctx.sender.clone()).await;

    if result.is_err() {
        bail!("sub: [start] open_channel call failed");
    }

    Ok(())
}
