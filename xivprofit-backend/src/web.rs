use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use futures::future::{BoxFuture, Shared};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::Mutex;

use crate::{
    cache::InMemoryCache,
    crafting::{Item, ItemData, Recipe},
    market::{self, ItemListing},
};

#[derive(Clone)]
pub(crate) struct Context {
    pub(crate) cache: Arc<InMemoryCache>,
    pub(crate) item_data: Arc<ItemData>,
    pub(crate) jobs: Arc<Mutex<HashMap<String, Shared<BoxFuture<'static, Vec<ItemListing>>>>>>,
}

#[derive(Deserialize)]
pub(crate) struct GetItemListingsRequest {
    item_id: usize,
    location: String,
}

#[derive(Deserialize)]
pub(crate) struct GetCheapestListingsRequest {
    item_id: usize,
    amount: usize,
    location: String,
    hq: bool,
}

pub(crate) async fn get_listings(
    State(context): State<Context>,
    r: Query<GetItemListingsRequest>,
) -> (StatusCode, Json<Vec<ItemListing>>) {
    let listings = market::get_item_listings(&r.location, r.item_id, &context.cache).await;
    (StatusCode::OK, Json(listings))
}

#[axum::debug_handler]
pub(crate) async fn get_saleprice(
    State(context): State<Context>,
    r: Query<GetItemListingsRequest>,
) -> (StatusCode, String) {
    let mut listings = market::get_item_listings(&r.location, r.item_id, &context.cache).await;
    listings.sort_by(|a, b| a.price_per_unit.total_cmp(&b.price_per_unit));
    let saleprice = match listings.first() {
        Some(l) => l.price_per_unit,
        None => 0.0,
    };
    (StatusCode::OK, saleprice.to_string())
}

pub(crate) async fn get_cheapest_listings(
    State(context): State<Context>,
    r: Query<GetCheapestListingsRequest>,
) -> (StatusCode, Json<Vec<ItemListing>>) {
    if r.amount < 1 || r.amount > 1000 {
        return (StatusCode::BAD_REQUEST, Json(Vec::new()));
    }
    if context
        .item_data
        .items
        .clone()
        .iter()
        .any(|i| i.id == r.item_id)
    {
        let listings = market::get_cheapest_combination(
            r.item_id,
            r.location.clone(),
            &context.cache,
            r.amount,
            r.hq,
            &context.jobs,
        )
        .await;
        (StatusCode::OK, Json(listings))
    } else {
        (StatusCode::BAD_REQUEST, Json(Vec::new()))
    }
}

pub(crate) async fn get_items(State(context): State<Context>) -> (StatusCode, Json<Vec<Item>>) {
    (StatusCode::OK, Json(context.item_data.items.clone()))
}

async fn get_all_recipes(context: Context) -> (StatusCode, Json<Vec<Recipe>>) {
    (StatusCode::OK, Json(context.item_data.recipes.clone()))
}

async fn get_recipes_for_item(context: Context, item_id: usize) -> (StatusCode, Json<Vec<Recipe>>) {
    let mut recipes = context.item_data.recipes.clone();
    recipes.retain(|r| r.result_item_id == item_id);
    (StatusCode::OK, Json(recipes))
}

pub(crate) async fn get_recipes(
    State(context): State<Context>,
    item_id: Option<Query<usize>>,
) -> (StatusCode, Json<Vec<Recipe>>) {
    match item_id {
        None => get_all_recipes(context).await,
        Some(id) => get_recipes_for_item(context, *id).await,
    }
}

pub(crate) async fn get_craftable_items(
    State(context): State<Context>,
) -> (StatusCode, Json<Vec<Item>>) {
    (
        StatusCode::OK,
        Json(context.item_data.craftable_items.clone()),
    )
}
