use std::{env, io};

use actix_web::web::Data;
use diesel::{backend::Backend};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{Connection, Identifiable};
use juniper::http::GraphQLRequest;
use juniper::LookAheadSelection;
use std::sync::Arc;
use wundergraph::error::Result as WunderResult;
use wundergraph::query_builder::selection::offset::ApplyOffset;
use wundergraph::query_builder::selection::{BoxedQuery, LoadingHandler, QueryModifier};
use wundergraph::scalar::WundergraphScalarValue;
use wundergraph::WundergraphContext;

use crate::generated::*;

pub type DbConnection = diesel::PgConnection;
pub type DbPool = Pool<ConnectionManager<DbConnection>>;
pub type DbManager<Conn> = PooledConnection<ConnectionManager<Conn>>;
pub type GraphQLData = GraphQLRequest<WundergraphScalarValue>;

#[derive(Debug)]
pub struct DbContext<Conn>
where
    Conn: Connection + 'static,
{
    pub conn: DbManager<Conn>,
}

impl<Conn> DbContext<Conn>
where
    Conn: Connection + 'static,
{
    pub fn new(conn: DbManager<Conn>) -> Self {
        Self { conn }
    }
}

impl<T, C, Db> QueryModifier<T, Db> for DbContext<C>
where
    C: Connection<Backend = Db>,
    Db: Backend + ApplyOffset + 'static,
    T: LoadingHandler<Db, Self>,
    Self: WundergraphContext,
    Self::Connection: Connection<Backend = Db>,
{
    fn modify_query<'a>(
        &self,
        _select: &LookAheadSelection<'_, WundergraphScalarValue>,
        query: BoxedQuery<'a, T, Db, Self>,
    ) -> WunderResult<BoxedQuery<'a, T, Db, Self>> {
        match T::TYPE_NAME {
            _ => Ok(query),
        }
    }
}

impl WundergraphContext for DbContext<DbConnection> {
    type Connection = DbManager<DbConnection>;

    fn get_connection(&self) -> &Self::Connection {
        &self.conn
    }
}

pub type Schema<Ctx> =
    juniper::RootNode<'static, Query<Ctx>, Mutation<Ctx>, WundergraphScalarValue>;

#[derive(Clone)]
pub struct AppState {
    pub schema: Arc<Schema<DbContext<DbConnection>>>,
    pub pool: Arc<DbPool>,
}

pub fn get_data(pool: DbPool) -> Data<AppState> {
    let query = Query::<DbContext<DbConnection>>::default();
    let mutation = Mutation::<DbContext<DbConnection>>::default();
    let schema = Schema::new(query, mutation);
    let schema = Arc::new(schema);
    let pool = Arc::new(pool);
    Data::new(AppState { schema, pool })
}

