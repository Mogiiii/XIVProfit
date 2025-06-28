use std::{
    env,
    sync::{Arc, RwLock},
    time::Instant,
};
use tracing::trace;

use crate::market::ItemListing;

use std::{collections::HashMap, string::String, time::Duration};

struct CacheValue {
    data: Vec<ItemListing>,
    expiration: Instant,
}

impl CacheValue {
    fn is_expired(&self) -> bool {
        return Instant::now() > self.expiration;
    }
}

pub(crate) struct InMemoryCache {
    mem: Arc<RwLock<HashMap<String, CacheValue>>>,
    cache_timeout: u64,
}

impl InMemoryCache {
    pub(crate) fn new() -> Self {
        Self {
            mem: Arc::new(RwLock::new(HashMap::new())),
            cache_timeout: env::var("XIVP_CACHE_TIMEOUT")
                .unwrap_or(String::from("300"))
                .parse()
                .unwrap_or(300),
        }
    }
}

impl InMemoryCache {
    pub(crate) fn get_listing(&self, item_id: usize, world: String) -> Option<Vec<ItemListing>> {
        let id = format!("listing-{world}-{item_id}");
        trace!("getting {id}");
        let store = self.mem.read().unwrap();
        if let Some(val) = store.get(&id) {
            if val.is_expired() {
                trace!("Cache expired: {id}");
                None
            } else {
                trace!("Cache hit: {id}");
                Some(val.data.clone())
            }
        } else {
            trace!("Cache miss: {id}");
            None
        }
    }

    pub(crate) fn set_listing(&self, item_id: usize, world: String, data: Vec<ItemListing>) -> () {
        let id = format!("listing-{world}-{item_id}");
        trace!("setting {id}");
        if let Some(_) = self.get_listing(item_id, world) {
            trace!("Race condition setting {id} :)")
        } else {
            self.mem.write().unwrap().insert(
                id.clone(),
                CacheValue {
                    data,
                    expiration: Instant::now() + Duration::from_secs(self.cache_timeout),
                },
            );
        }
    }

    pub(crate) fn get_cheapest(
        &self,
        item_id: usize,
        world: String,
        amount: usize,
        hq: bool,
    ) -> Option<Vec<ItemListing>> {
        let id = format!("cheapest-{world}-{item_id}-{amount}-{hq}");
        trace!("getting {id}");
        let store = self.mem.read().unwrap();
        if let Some(val) = store.get(&id) {
            if val.is_expired() {
                trace!("Cache expired: {id}");
                None
            } else {
                trace!("Cache hit: {id}");
                Some(val.data.clone())
            }
        } else {
            trace!("Cache miss: {id}");
            None
        }
    }

    pub(crate) fn set_cheapest(
        &self,
        item_id: usize,
        world: String,
        amount: usize,
        hq: bool,
        data: Vec<ItemListing>,
    ) -> () {
        let id = format!("cheapest-{world}-{item_id}-{amount}-{hq}");
        trace!("setting {id}");
        if let Some(_) = self.get_cheapest(item_id, world, amount, hq) {
            trace!("Race condition setting {id}")
        } else {
            self.mem.write().unwrap().insert(
                id.clone(),
                CacheValue {
                    data,
                    expiration: Instant::now() + Duration::from_secs(3600),
                },
            );
        }
    }
}
