use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Represents the cached response, containing the response body and the timestamp of when it was cached.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct CachedResponse {
    body: String,
    timestamp: Instant,
}

/// The web cache structure that stores responses, using a HashMap for fast retrieval.
struct WebCache {
    cache: Mutex<HashMap<String, CachedResponse>>,
    client: Client,
    ttl: Duration,
    capacity: usize,
    history: Mutex<VecDeque<String>>, // Stores the history of URLs fetched.
    cleanup_running: Mutex<bool>,
}

impl WebCache {
    /// Creates a new `WebCache` instance with the specified time-to-live (TTL) for cached responses and cache capacity.
    fn new(ttl: Duration, capacity: usize) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            client: Client::new(),
            ttl,
            capacity,
            history: Mutex::new(VecDeque::new()),
            cleanup_running: Mutex::new(false),
        }
    }

    /// Fetches a URL, using the cache if the response is still valid, or fetching from the web otherwise.
    fn fetch(&self, url: &str) -> String {
        let mut cache = self.cache.lock().unwrap();
        let mut history = self.history.lock().unwrap();

        if let Some(cached_response) = cache.get(url) {
            if cached_response.timestamp.elapsed() < self.ttl {
                history.push_back(url.to_string());
                if history.len() > self.capacity {
                    history.pop_front();
                }
                return cached_response.body.clone();
            }
        }

        let response_body = self.client.get(url).send().unwrap().text().unwrap();
        cache.insert(
            url.to_string(),
            CachedResponse {
                body: response_body.clone(),
                timestamp: Instant::now(),
            },
        );

        if cache.len() > self.capacity {
            self.evict_oldest_entry(&mut cache);
        }

        history.push_back(url.to_string());
        if history.len() > self.capacity {
            history.pop_front();
        }

        response_body
    }

    /// Evicts the oldest entry from the cache to maintain the specified capacity.
    fn evict_oldest_entry(&self, cache: &mut HashMap<String, CachedResponse>) {
        if let Some(oldest_key) = cache
            .iter()
            .min_by_key(|(_, response)| response.timestamp)
            .map(|(key, _)| key.clone())
        {
            cache.remove(&oldest_key);
        }
    }

    /// Periodically clears stale cache entries. This method ensures that the cleanup thread runs only once.
    fn start_cleanup_thread(self: Arc<Self>) {
        let mut running = self.cleanup_running.lock().unwrap();
        if *running {
            return;
        }

        *running = true;
        drop(running);

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(60));
            let mut cache = self.cache.lock().unwrap();
            cache.retain(|_, response| response.timestamp.elapsed() < self.ttl);
        });
    }

    /// Retrieves the current size of the cache.
    fn cache_size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Retrieves the time-to-live (TTL) of the cache.
    fn get_ttl(&self) -> Duration {
        self.ttl
    }

    /// Sets a new time-to-live (TTL) for the cache. This will affect all new entries.
    fn set_ttl(&mut self, new_ttl: Duration) {
        self.ttl = new_ttl;
    }

    /// Retrieves the cache capacity, i.e., the maximum number of entries the cache can hold.
    fn get_capacity(&self) -> usize {
        self.capacity
    }

    /// Sets a new cache capacity. If the current cache size exceeds the new capacity, oldest entries are evicted.
    fn set_capacity(&mut self, new_capacity: usize) {
        let mut cache = self.cache.lock().unwrap();
        self.capacity = new_capacity;
        while cache.len() > self.capacity {
            self.evict_oldest_entry(&mut cache);
        }
    }

    /// Retrieves the cache history as a vector of URLs.
    fn get_history(&self) -> Vec<String> {
        let history = self.history.lock().unwrap();
        history.iter().cloned().collect()
    }

    /// Clears the entire cache and history.
    fn clear_cache_and_history(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut history = self.history.lock().unwrap();
        cache.clear();
        history.clear();
    }

    /// Serializes the cache to a JSON string.
    fn get_cache_as_json(&self) -> String {
        let cache = self.cache.lock().unwrap();
        serde_json::to_string(&*cache).unwrap()
    }

    /// Restores the cache from a JSON string.
    fn load_cache_from_json(&self, json: &str) {
        let mut cache = self.cache.lock().unwrap();
        let deserialized: HashMap<String, CachedResponse> =
            serde_json::from_str(json).unwrap();
        *cache = deserialized;
    }

    /// Saves the current cache to a file.
    fn save_cache_to_file(&self, path: &str) {
        let json = self.get_cache_as_json();
        std::fs::write(path, json).unwrap();
    }

    /// Loads the cache from a file.
    fn load_cache_from_file(&self, path: &str) {
        if let Ok(json) = std::fs::read_to_string(path) {
            self.load_cache_from_json(&json);
        }
    }

    /// Retrieves the oldest cached entry.
    fn get_oldest_entry(&self) -> Option<(String, CachedResponse)> {
        let cache = self.cache.lock().unwrap();
        cache.iter().min_by_key(|(_, response)| response.timestamp).map(|(key, value)| (key.clone(), value.clone()))
    }

    /// Retrieves the newest cached entry.
    fn get_newest_entry(&self) -> Option<(String, CachedResponse)> {
        let cache = self.cache.lock().unwrap();
        cache.iter().max_by_key(|(_, response)| response.timestamp).map(|(key, value)| (key.clone(), value.clone()))
    }

    /// Checks if a specific URL is in the cache.
    fn is_cached(&self, url: &str) -> bool {
        let cache = self.cache.lock().unwrap();
        cache.contains_key(url)
    }
}

/// Utility function to format a duration in a human-readable way (e.g., "2h 30m 5s").
fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{}h {}m {}s", hours, minutes, seconds)
}

/// Utility function to calculate the total size of the cache in bytes.
fn calculate_cache_size_in_bytes(cache: &HashMap<String, CachedResponse>) -> usize {
    cache.iter().map(|(k, v)| k.len() + v.body.len()).sum()
}

/// Clears the cache completely.
fn clear_cache(cache: &Mutex<HashMap<String, CachedResponse>>) {
    let mut cache = cache.lock().unwrap();
    cache.clear();
}

/// Load test function to simulate load by fetching the same URL multiple times in parallel threads.
fn load_test_cache(cache: Arc<WebCache>, url: &str, num_threads: usize) {
    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            let cache = Arc::clone(&cache);
            let url = url.to_string();
            thread::spawn(move || {
                for _ in 0..100 {
                    cache.fetch(&url);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}