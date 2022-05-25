use std::env;

use actix_web::{
    middleware, web,
    web::{Data, Json},
    App, Error as ActixError, HttpResponse, HttpServer,
};
use diesel::backend::Backend;
use diesel::r2d2::{ConnectionManager, Pool};
use juniper::http::playground::playground_source;
use std::sync::Arc;

use crate::generated::*;
use crate::db::*;


async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source("/graphql"))
}

async fn graphql(
    Json(data): Json<GraphQLData>,
    st: Data<AppState>,
) -> Result<HttpResponse, ActixError> {
    let ctx = DbContext::new(st.get_ref().pool.get().expect("Fail to get pool"));
    let res = data.execute(&st.get_ref().schema, &ctx);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&res)?))
}

async fn upload_logs() -> Result<HttpResponse, ActixError> {
    todo!()
}

pub fn configure_endpoints(cfg: &mut web::ServiceConfig) {
    cfg.route("/graphql", web::get().to(graphql_playground))
        .route("/graphql", web::post().to(graphql));
    cfg.service(web::resource("/test")
        .route(web::get().to(|| HttpResponse::Ok()))
        .route(web::head().to(|| HttpResponse::MethodNotAllowed()))
    );
}
