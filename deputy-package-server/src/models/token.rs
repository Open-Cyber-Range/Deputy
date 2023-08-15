use crate::{models::helpers::uuid::Uuid, schema::tokens};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Deserialize, Serialize)]
#[diesel(table_name = tokens)]
pub struct ApiToken {
    pub id: Uuid,
    pub name: String,
    pub token: String,
    pub user_id: String,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Deserialize, Serialize)]
#[diesel(table_name = tokens)]
pub struct NewApiToken {
    pub id: Uuid,
    pub name: String,
    pub token: String,
    pub user_id: String,
}
