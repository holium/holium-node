use anyhow::Result;
use serde_json::Value as JsonValue;

pub async fn query_messages(_timestamp: i64) -> Result<JsonValue> {
    Ok(JsonValue::Null)
}
