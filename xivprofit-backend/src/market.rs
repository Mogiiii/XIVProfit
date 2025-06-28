use futures::{
    future::{BoxFuture, Shared},
    FutureExt,
};
use log::trace;
use reqwest::{Error, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Mutex;

mod optimizer;

use crate::cache::InMemoryCache;

#[derive(Clone, Serialize)]
pub(crate) struct ItemListing {
    item_id: usize,
    world_id: usize,
    pub(crate) price_per_unit: f32,
    pub(crate) quantity: usize,
    pub(crate) total_price: usize,
    hq: bool,
    retainer_name: String, //or npc vendor name
}

//response from https://universalis.app/api/v2/{{world}}/{{itemid}}
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub(crate) struct UniversalisMbCurrent {
    itemID: usize,
    worldID: Option<usize>,
    //lastUploadTime: usize,
    listings: Vec<UniversalisMbListing>,
    // currentAveragePrice: f32,
    // currentAveragePriceNQ: f32,
    // currentAveragePriceHQ: f32,
    // regularSaleVelocity: f32,
    // nqSaleVelocity: f32,
    // hqSaleVelocity: f32,
    // averagePrice: f32,
    // averagePriceNQ: f32,
    // averagePriceHQ: f32,
    // minPrice: usize,
    // minPriceNQ: usize,
    // minPriceHQ: usize,
    // stackSizeHistogram
    // stackSizeHistogramNQ
    // stackSizeHistogramHQ
    // worldName: String,
    // listingsCount: usize,
    // recentHistoryCount: usize,
    // unitsForSale: usize,
    // unitsSold: usize,
}
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct UniversalisMbListing {
    worldID: Option<usize>,
    pricePerUnit: f32,
    quantity: usize,
    hq: bool,
    retainerName: String,
    total: usize,
}

impl UniversalisMbCurrent {
    pub(crate) fn to_listings(self) -> Vec<ItemListing> {
        let mut v = Vec::new();
        for l in self.listings {
            let id = match self.worldID {
                None => l.worldID.unwrap(),
                Some(i) => i,
            };

            v.push(ItemListing {
                hq: l.hq,
                item_id: self.itemID,
                world_id: id,
                price_per_unit: l.pricePerUnit,
                quantity: l.quantity,
                total_price: l.total,
                retainer_name: l.retainerName,
            });
        }
        v
    }
}
pub(crate) async fn get_universalis_mb_data(
    location: &String,
    item_id: usize,
) -> Result<UniversalisMbCurrent, Error> {
    let base_url = env::var("XIVP_UNIVERSALIS_API").expect("Missing Env var: XIVP_UNIVERSALIS_API");
    //let base_url = "https://universalis.app/api/v2/";
    trace!("getting universalis data for {item_id} @ {location}");
    let client = reqwest::Client::new();

    let r = client
        .get(format!("{base_url}/{location}/{item_id}"))
        .send()
        .await?;
    match r.status() {
        StatusCode::OK => Ok(r.json().await?),
        _ => Err(r
            .error_for_status()
            .expect_err("no error when expecting error")),
    }
}

pub(crate) async fn get_item_listings(
    world: &String,
    item_id: usize,
    cache: &InMemoryCache,
) -> Vec<ItemListing> {
    if let Some(v) = cache.get_listing(item_id, world.clone()) {
        v
    } else {
        let data = get_universalis_mb_data(world, item_id)
            .await
            .unwrap()
            .to_listings();
        cache.set_listing(item_id, world.clone(), data.clone());
        data
    }
}

//only allow 1 thread to run optimizer::get_cheapest_combination for a set of arguments at a time, all others should just wait for that one and return the same result
pub(crate) async fn get_cheapest_combination(
    item_id: usize,
    location: String,
    cache: &Arc<InMemoryCache>,
    amount: usize,
    hq: bool,
    running_jobs: &Arc<Mutex<HashMap<String, Shared<BoxFuture<'static, Vec<ItemListing>>>>>>,
) -> Vec<ItemListing> {

    let mut running = running_jobs.lock().await;
    let id = format!("cheapest-{location}-{item_id}-{amount}-{hq}");
    if let Some(r) = running.get(&id) {
        //another thread is running this
        return r.clone().await;
    }

    let c = cache.clone();
    let l = location.clone();
    //another thread is not running, and we should run
    let fut = async move { optimizer::get_cheapest_combination(item_id, l, &c, amount, hq).await }
        .boxed()
        .shared();
    running.insert(id.clone(), fut.clone());
    drop(running); //release mutex

    let r = fut.await;
    running_jobs.lock().await.remove(&id);

    r
}
