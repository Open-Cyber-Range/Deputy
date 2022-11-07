pub(crate) mod package;

use actix::Actor;
use anyhow::{anyhow, Result};
use diesel::{
    mysql::{Mysql, MysqlConnection},
    helper_types::{AsSelect, Filter, IsNull, Select},
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

pub type All<Table, T> = Select<Table, AsSelect<T, Mysql>>;
pub type FilterExisting<Target, DeletedAtColumn> = Filter<Target, IsNull<DeletedAtColumn>>;


pub struct Database {
    connection_pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Actor for Database {
    type Context = actix::Context<Self>;
}

impl Database {
    pub fn try_new(database_url: &str) -> Result<Self> {
        let manager = ConnectionManager::<MysqlConnection>::new(database_url);
        Ok(Self {
            connection_pool: Pool::builder()
                .build(manager)
                .map_err(|error| anyhow!("Failed to create database connection pool: {}", error))?,
        })
    }

    pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<MysqlConnection>>> {
        Ok(self.connection_pool.get()?)
    }
}
