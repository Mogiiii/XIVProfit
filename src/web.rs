use std::sync::Arc;

use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    cache::InMemoryCache,
    crafting::{calc_profit, Item, ItemData, Recipe},
    market::{self, ItemListing},
};

#[derive(Clone)]
pub(crate) struct Context {
    pub(crate) cache: Arc<InMemoryCache>,
    pub(crate) item_data: Arc<ItemData>,
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
}

#[derive(Deserialize)]
pub(crate) struct GetProfitRequest {
    recipe_id: usize,
    location: String,
    amount: usize,
}

#[derive(Serialize)]
pub(crate) struct GetProfitResponse {
    profit: f32,
    items: Vec<ItemListing>,
}

pub(crate) async fn get_item_listings(
    State(context): State<Context>,
    r: Query<GetItemListingsRequest>,
) -> (StatusCode, Json<Vec<ItemListing>>) {
    let listings = market::get_item_listings(&r.location, r.item_id, &context.cache).await;
    (StatusCode::OK, Json(listings))
}

pub(crate) async fn get_cheapest_listings(
    State(context): State<Context>,
    r: Query<GetCheapestListingsRequest>,
) -> (StatusCode, Json<Vec<ItemListing>>) {
    if context.item_data.items.clone().iter().any( |i| i.id == r.item_id) {
    let listings = market::get_cheapest_combination(r.item_id, &r.location, &context.cache, r.amount).await;
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

pub(crate) async fn get_profit(
    State(context): State<Context>,
    req: Query<GetProfitRequest>,
) -> (StatusCode, Json<Option<GetProfitResponse>>) {
    match calc_profit(
        req.recipe_id,
        req.location.clone(),
        req.amount,
        &context.item_data,
        &context.cache,
    )
    .await
    {
        Ok((cost, listings)) => {
            let r = GetProfitResponse {
                profit: cost,
                items: listings,
            };
            (StatusCode::OK, Json(Some(r)))
        }
        Err(_) => (StatusCode::NOT_FOUND, Json(None)),
    }
}

pub(crate) async fn get_craftable_items(State(context): State<Context>) -> (StatusCode, Json<Vec<Item>>) {
    (StatusCode::OK, Json(context.item_data.craftable_items.clone()))
}