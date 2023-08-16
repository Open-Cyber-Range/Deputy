use crate::{
    models::helpers::uuid::Uuid,
    schema::tokens::{self},
    services::database::{All, Create, FilterExisting},
};
use base64::{engine::general_purpose, Engine as _};
use chrono::NaiveDateTime;
use diesel::{helper_types::FindBy, insert_into, prelude::*};
use rand::Rng;
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

impl ApiToken {
    pub fn by_id(
        id: Uuid,
    ) -> FindBy<FilterExisting<All<tokens::table, Self>, tokens::deleted_at>, tokens::id, Uuid>
    {
        Self::all().filter(tokens::id.eq(id))
    }

    fn all_with_deleted() -> All<tokens::table, Self> {
        tokens::table.select(Self::as_select())
    }

    pub fn all() -> FilterExisting<All<tokens::table, Self>, tokens::deleted_at> {
        Self::all_with_deleted().filter(tokens::deleted_at.is_null())
    }
}

#[derive(Insertable, Deserialize, Serialize)]
#[diesel(table_name = tokens)]
pub struct NewApiToken {
    pub id: Uuid,
    pub name: String,
    pub token: String,
    pub user_id: String,
}

impl NewApiToken {
    pub fn create_insert(&self) -> Create<&Self, tokens::table> {
        insert_into(tokens::table).values(self)
    }
}

impl From<(String, String)> for NewApiToken {
    fn from((name, user_id): (String, String)) -> Self {
        Self {
            id: Uuid::random(),
            name,
            token: generate_token(),
            user_id,
        }
    }
}

pub fn generate_token() -> String {
    let token_byte_length = 128;
    let random_bytes = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(token_byte_length)
        .collect::<Vec<u8>>();
    general_purpose::STANDARD.encode(random_bytes)
}
