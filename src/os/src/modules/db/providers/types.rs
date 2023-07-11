use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::modules::db::Db;

#[async_trait]
pub trait DataProvider {
    fn new(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
    async fn import_data(&self, db: &Db, options: Option<HashMap<&str, String>>) -> Result<()>;
}
