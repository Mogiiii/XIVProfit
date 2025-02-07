use std::borrow::Borrow;

use log::info;
use reqwest::{Error, StatusCode};
use serde::{Deserialize, Serialize};

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
    // hasData: bool,
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
    world: &String,
    item_id: usize,
) -> Result<UniversalisMbCurrent, Error> {
    //let base_url = env::var("UNIVERSALIS_API").expect("Missing Env var: UNIVERSALIS_API");
    let base_url = "https://universalis.app/api/v2/";
    info!("getting universalis data for {item_id} on {world}");
    let client = reqwest::Client::new();

    let r = client
        .get(format!("{base_url}/{world}/{item_id}"))
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
    if let Some(v) = cache.get(item_id, world.clone()) {
        v
    } else {
        let data = get_universalis_mb_data(world, item_id)
            .await
            .unwrap()
            .to_listings();
        cache.set(item_id, world.clone(), data.clone());
        data
    }
}

pub(crate) async fn get_cheapest_combination(
    item_id: usize,
    world: &String,
    cache: &InMemoryCache,
    amount: usize,
) -> Vec<ItemListing> {
    let mut listings = get_item_listings(world, item_id, cache).await;
    listings.sort_by(|fst, snd| fst.total_price.cmp(snd.total_price.borrow()));
    let mut cheapest: Option<Vec<ItemListing>> = None;

    struct CombinationIterator {
        data: Vec<ItemListing>,
        index: usize,
        size: usize,
        max_size: usize,
    }

    impl CombinationIterator {
        fn from(listings: Vec<ItemListing>) -> Self {
            Self {
                data: listings.clone(),
                index: 0,
                size: 1,
                max_size: listings.len() - 1,
            }
        }
    }

    impl Iterator for CombinationIterator {
        fn next(&mut self) -> Option<Vec<ItemListing>> {
            if self.index > self.max_size {
                self.size = self.size + 1;
                self.index = self.size;
            }
            if self.size > self.max_size {
                return None;
            }
            let mut i: usize = 0;
            let mut r = Vec::new();

            while i < self.size && i != self.index {
                r.push(self.data[i].clone());
                i = i + 1;
            }
            r.push(self.data[self.index].clone());
            self.index += 1;
            Some(r)
        }

        type Item = Vec<ItemListing>;
    }

    for listing_combination in CombinationIterator::from(listings.clone()) {
        let mut amt = 0;
        let mut cost = 0;
        for listing in &listing_combination {
            amt += listing.quantity;
            cost += listing.total_price;
        }
        if amt >= amount {
            match cheapest {
                None => cheapest = Some(listing_combination.clone()),

                Some(ref combo) => {
                    let current_cheapest_price: usize =
                        combo.clone().iter().map(|l| l.total_price).sum();
                    if cost < current_cheapest_price {
                        cheapest = Some(combo.to_owned());
                    }
                }
            }
        }
    }
    if cheapest.is_some() {
        cheapest.unwrap()
    } else {
        Vec::new()
    }
}

pub(crate) async fn minimum_price_per_unit(
    item_id: usize,
    world: &String,
    cache: &InMemoryCache,
) -> f32 {
    let mut listings = get_item_listings(world, item_id, cache).await;
    {
        match listings.is_empty() {
            true => 0.,
            false => {
                listings
                    .sort_by(|fst, snd| fst.price_per_unit.total_cmp(snd.price_per_unit.borrow()));
                listings.first().expect("msg").price_per_unit
            }
        }
    }
}
