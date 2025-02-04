use log::debug;
use std::{
    sync::{Arc, RwLock},
    time::Instant,
};
use tracing::info;

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
}

impl InMemoryCache {
    pub(crate) fn new() -> Self {
        Self {
            mem: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl InMemoryCache {
    pub(crate) fn get(&self, item_id: usize, world: String) -> Option<Vec<ItemListing>> {
        let id = format!("{world}-{item_id}");
        info!("getting {id}");
        let store = self.mem.read().unwrap();
        if let Some(val) = store.get(&id) {
            if val.is_expired() {
                info!("Cache expired: {id}");
                None
            } else {
                info!("Cache hit: {id}");
                Some(val.data.clone())
            }
        } else {
            info!("Cache miss: {id}");
            None
        }
    }

    pub(crate) fn set(&self, item_id: usize, world: String, data: Vec<ItemListing>) -> () {
        let id = format!("{world}-{item_id}");
        info!("setting {id}");
        if let Some(_) = self.get(item_id, world) {
            debug!("Race condition setting {id} :)")
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