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

#[derive(Queryable, Selectable, Eq, PartialEq, Deserialize, Serialize, Clone, Debug)]
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
    fn all_with_deleted() -> All<tokens::table, Self> {
        tokens::table.select(Self::as_select())
    }

    pub fn all() -> FilterExisting<All<tokens::table, Self>, tokens::deleted_at> {
        Self::all_with_deleted().filter(tokens::deleted_at.is_null())
    }

    pub fn by_id(
        id: Uuid,
    ) -> FindBy<FilterExisting<All<tokens::table, Self>, tokens::deleted_at>, tokens::id, Uuid>
    {
        Self::all().filter(tokens::id.eq(id))
    }

    pub fn by_token(
        token: String,
    ) -> FindBy<FilterExisting<All<tokens::table, Self>, tokens::deleted_at>, tokens::token, String>
    {
        Self::all().filter(tokens::token.eq(token))
    }

    pub fn by_user_id(
        user_id: String,
    ) -> FindBy<FilterExisting<All<tokens::table, Self>, tokens::deleted_at>, tokens::user_id, String>
    {
        Self::all().filter(tokens::user_id.eq(user_id))
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiTokenRest {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

impl From<ApiToken> for ApiTokenRest {
    fn from(api_token: ApiToken) -> Self {
        Self {
            id: api_token.id,
            name: api_token.name,
            created_at: api_token.created_at,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullApiTokenRest {
    pub id: Uuid,
    pub name: String,
    pub token: String,
    pub user_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<ApiToken> for FullApiTokenRest {
    fn from(api_token: ApiToken) -> Self {
        Self {
            id: api_token.id,
            name: api_token.name,
            token: api_token.token,
            user_id: api_token.user_id,
            created_at: api_token.created_at,
            updated_at: api_token.deleted_at,
        }
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

#[derive(Deserialize, Serialize)]
pub struct NewApiTokenRest {
    pub name: String,
}

impl NewApiTokenRest {
    fn generate_token() -> String {
        let token_byte_length = 128;
        let random_bytes = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(token_byte_length)
            .collect::<Vec<u8>>();
        general_purpose::STANDARD.encode(random_bytes)
    }

    pub fn create_new_token(&self, user_id: String) -> NewApiToken {
        NewApiToken {
            id: Uuid::random(),
            name: self.name.clone(),
            token: Self::generate_token(),
            user_id,
        }
    }
}
