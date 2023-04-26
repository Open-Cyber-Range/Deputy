pub(crate) mod package;

use crate::models::helpers::uuid::Uuid;
use crate::utilities::run_migrations;
use actix::Actor;
use anyhow::{anyhow, Result};
use diesel::{
    helper_types::{AsSelect, Eq, Filter, IsNull, Select},
    mysql::{Mysql, MysqlConnection},
    query_builder::InsertStatement,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    Insertable,
};

pub type All<Table, T> = Select<Table, AsSelect<T, Mysql>>;
pub type FilterExisting<Target, DeletedAtColumn> = Filter<Target, IsNull<DeletedAtColumn>>;
pub type ById<Id, R> = Filter<R, Eq<Id, Uuid>>;
pub type SelectById<Table, Id, DeletedAtColumn, T> =
    ById<Id, FilterExisting<All<Table, T>, DeletedAtColumn>>;
pub type Create<Type, Table> = InsertStatement<Table, <Type as Insertable<Table>>::Values>;

#[derive(Clone)]
pub struct Database {
    connection_pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl Actor for Database {
    type Context = actix::Context<Self>;
}

impl Database {
    pub fn try_new(database_url: &str) -> Result<Self> {
        let manager = ConnectionManager::<MysqlConnection>::new(database_url);
        let connection_pool = Pool::builder()
            .build(manager)
            .map_err(|error| anyhow!("Failed to create database connection pool: {}", error))?;
        let mut connection = connection_pool
            .get()
            .map_err(|error| anyhow!("Failed to get database connection: {}", error))?;
        run_migrations(&mut connection)
            .map_err(|error| anyhow!("Failed to run database migrations: {}", error))?;
        Ok(Self { connection_pool })
    }

    pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<MysqlConnection>>> {
        Ok(self.connection_pool.get()?)
    }
}
