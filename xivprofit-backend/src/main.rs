mod cache;
mod crafting;
mod market;
mod web;

use cache::InMemoryCache;
use crafting::ItemData;
use dotenvy::dotenv;
use log::info;
use std::{collections::HashMap, env};
use std::sync::Arc;

use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // initialize tracing
    tracing_subscriber::fmt::init();
    let ctx = web::Context {
        cache: Arc::new(InMemoryCache::new()),
        item_data: Arc::new(ItemData::new().await),
        jobs: Arc::new(HashMap::new().into()),
    };

    // build our application with a route
    let app = Router::new()
        .route("/api/listings", get(web::get_listings))
        .route("/api/items", get(web::get_items))
        .route("/api/recipes", get(web::get_recipes))
        //.route("/api/profit", get(web::get_profit))
        .route("/api/craftable_items", get(web::get_craftable_items))
        .route("/api/cheapestlistings", get(web::get_cheapest_listings))
        .route("/api/saleprice", get(web::get_saleprice))
        .with_state(ctx)
        .layer(CorsLayer::permissive());

    let host = env::var("XIVP_HTTP_HOST").expect("Missing Env var: XIVP_HTTP_HOST");
    let port = env::var("XIVP_HTTP_PORT").expect("Missing Env var: XIVP_HTTP_PORT");
    info!("Starting webserver on {host}:{port}");
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
