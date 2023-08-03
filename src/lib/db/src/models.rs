use chrono::prelude::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::passports)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Passport {
    pub ship: String,
    pub is_public: bool,
    pub nickname: String,
    pub color: String,
    pub twitter: Option<String>,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub cover: Option<String>,
    pub featured_url: Option<String>,
    pub phone_number: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
