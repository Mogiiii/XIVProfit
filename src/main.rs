mod cache;
mod crafting;
mod market;
mod web;

use cache::InMemoryCache;
use crafting::ItemData;
use dotenvy::dotenv;
use log::info;
use std::env;
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
    };

    // build our application with a route
    let app = Router::new()
        .route("/listings", get(web::get_item_listings))
        .route("/items", get(web::get_items))
        .route("/recipes", get(web::get_recipes))
        .route("/profit", get(web::get_profit))
        .route("/craftable_items", get(web::get_craftable_items))
        .route("/cheapestlistings", get(web::get_cheapest_listings))
        .with_state(ctx)
        .layer(CorsLayer::permissive());

    let host = env::var("HTTP_HOST").expect("Missing Env var: HTTP_HOST");
    let port = env::var("HTTP_PORT").expect("Missing Env var: HTTP_PORT");
    info!("Starting webserver on {host}:{port}");
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
