use crate::{
    cache::InMemoryCache,
    market::{get_item_listings, ItemListing},
};
use itertools::Itertools;
use log::trace;
use std::time::SystemTime;

pub(super) async fn get_cheapest_combination(
    item_id: usize,
    location: String,
    cache: &InMemoryCache,
    amount: usize,
    hq: bool,
) -> Vec<ItemListing> {
    let listings = get_item_listings(&location, item_id, &cache).await;
    fn compute(
        listings: Vec<ItemListing>,
        item_id: usize,
        location: &String,
        amount: usize,
        hq: bool,
    ) -> Option<Vec<ItemListing>> {
        let mut cheapest: Option<Vec<ItemListing>> = None;

        let listing_count = listings.clone().iter().count();
        let mut loop_count: i64 = 0;
        trace!(
        "getting cheapest item_id:{item_id} location:{location} amount:{amount} hq: {hq} listings count: {listing_count}"
    );
        let start_time = SystemTime::now();
        for i in 1..listing_count {
            //quit when the next search space is massive
            if listing_count.pow(i as u32) > 10_usize.pow(8) {
                //trace!("stopping search dude to huge search space");
                break;
            }
            //if we have a potential solution already
            if let Some(ref c) = cheapest {
                //ignore solutions > size of current best + 1
                if i > c.len() + 1 {
                    break;
                }
            }
            for listing_combination in listings.iter().combinations(i) {
                loop_count += 1;
                let mut amt = 0;
                let mut cost = 0;
                for listing in listing_combination.clone() {
                    amt += listing.quantity;
                    cost += listing.total_price;
                }
                if amt >= amount {
                    match cheapest {
                        None => {
                            cheapest = {
                                let mut new_cheapest = Vec::new();
                                for listing in listing_combination.clone() {
                                    new_cheapest.push(listing.clone());
                                }
                                Some(new_cheapest)
                            }
                        }

                        Some(ref old_cheapest) => {
                            let current_cheapest_price: usize =
                                old_cheapest.clone().iter().map(|l| l.total_price).sum();
                            if cost < current_cheapest_price {
                                let mut new_cheapest = Vec::new();
                                for listing in listing_combination.clone() {
                                    new_cheapest.push(listing.clone());
                                }
                                cheapest = Some(new_cheapest.to_owned());
                            }
                        }
                    }
                };
            }
        }
        trace!(
        "finished item_id:{item_id} location:{location} amount:{amount} hq: {hq} in {:?} after {loop_count} iterations",
        start_time.elapsed().unwrap()
        );
        cheapest
    }
    
    //check cache
    if let Some(v) = cache.get_cheapest(item_id, location.clone(), amount, hq) {
        return v;
    }
    //cache miss
    if hq {
        let hq_listings = listings.iter().filter(|l| l.hq == true).cloned().collect();
        match compute(hq_listings, item_id, &location, amount, hq) {
            Some(c) => {
                //update cache
                cache.set_cheapest(item_id, location.clone(), amount, hq, c.clone());
                c
            }
            None => {
                trace!("No HQ combinations found, trying any combination: item_id:{item_id} location:{location} amount:{amount}");
                let c = compute(listings, item_id, &location, amount, false).unwrap_or_default();

                //update both hq and nq cache as they evaluated to be the same
                cache.set_cheapest(item_id, location.clone(), amount, hq, c.clone());
                cache.set_cheapest(item_id, location.clone(), amount, false, c.clone());
                c
            }
        }
    } else {
        let c = compute(listings, item_id, &location, amount, hq).unwrap_or_default();
        cache.set_cheapest(item_id, location.clone(), amount, hq, c.clone());
        c
    }
}
