fn main() {
    println!("Hello, world!");
}

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

// Tipo per la funzione di caricamento dal backend
type DataLoader<K, V> = dyn Fn(&K) -> Result<V, String> + Send + Sync;

pub struct CacheManager<K, V> {
    stats: Mutex<CacheStats>,
    cache: Mutex<HashMap<K, (V, Instant)>>,
    default_ttl: Duration,
    max_capacity: usize,
    loader: Box<DataLoader<K, V>>,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub entries_count: usize,
}

impl<K, V> CacheManager<K, V>
where
    K: Clone + Hash + Eq + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Crea un nuovo CacheManager con TTL di default e capacità massima
    pub fn new(default_ttl: Duration, max_capacity: usize) -> Self {
        return CacheManager {
            stats: Mutex::new(CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                entries_count: 0,
            }),
            cache: Mutex::new(HashMap::new()),
            default_ttl: default_ttl,
            max_capacity: max_capacity,
            loader: Box::new(|_| Err("No loader configured".to_string())),
        };
    }

    /// Crea un nuovo CacheManager con funzione di caricamento dal backend
    pub fn with_loader(
        default_ttl: Duration,
        max_capacity: usize,
        loader: Box<DataLoader<K, V>>,
    ) -> Self {
        return CacheManager {
            stats: Mutex::new(CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                entries_count: 0,
            }),
            cache: Mutex::new(HashMap::new()),
            default_ttl: default_ttl,
            max_capacity: max_capacity,
            loader,
        };
    }

    /// Inserisce un valore nella cache con TTL di default
    pub fn put(&self, key: K, value: V) -> Result<(), String> {
        let mut cache = self.cache.lock().unwrap();
        if cache.len() >= self.max_capacity && !cache.contains_key(&key) {
            let mut stats = self.stats.lock().unwrap();
            stats.evictions += 1;
            return Err("Cache is full".to_string());
        }
        let expiration = Instant::now() + self.default_ttl;
        let is_new = cache.insert(key, (value, expiration)).is_none();
        if is_new {
            let mut stats = self.stats.lock().unwrap();
            stats.entries_count += 1;
        }
        Ok(())
    }

    /// Inserisce un valore nella cache con TTL personalizzato
    pub fn put_with_ttl(&self, key: K, value: V, ttl: Duration) -> Result<(), String> {
        let mut cache = self.cache.lock().unwrap();
        if cache.len() >= self.max_capacity && !cache.contains_key(&key) {
            return Err("Cache is full".to_string());
        }
        let expiration = Instant::now() + ttl;
        let is_new = cache.insert(key, (value, expiration)).is_none();
        if is_new {
            let mut stats = self.stats.lock().unwrap();
            stats.entries_count += 1;
        }
        Ok(())
    }

    /// Recupera un valore dalla cache
    /// Se non presente e il loader è configurato, tenta di caricarlo dal backend
    pub fn get(&self, key: &K) -> Result<Option<V>, String> {
        let cache = self.cache.lock().unwrap();
        if let Some(val) = cache.get(key) {
            let (ref v, _instant) = *val;
            let mut stats = self.stats.lock().unwrap();
            stats.hits += 1;
            return Ok(Some(v.clone()));
        } else {
            let mut stats = self.stats.lock().unwrap();
            stats.misses += 1;
            drop(stats); // Release the stats lock before calling the loader
            match (self.loader)(key) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(e),
            }
        }
    }

    /// Recupera un valore dalla cache senza utilizzare il loader
    pub fn get_cached_only(&self, key: &K) -> Option<V> {
        let cache = self.cache.lock().unwrap();
        if let Some(val) = cache.get(key) {
            let (ref v, _instant) = *val;
            let mut stats = self.stats.lock().unwrap();
            stats.hits += 1;
            return Some(v.clone());
        } else {
            return None
        }
    }

    /// Rimuove un valore dalla cache
    pub fn remove(&self, key: &K) -> bool {
        let mut cache = self.cache.lock().unwrap();
        if let Some(_val) = cache.get(key) {
            cache.remove(key);
            let mut stats = self.stats.lock().unwrap();
            stats.entries_count -= 1;
            return true
        } else {
            return false
        }
    }

    /// Invalida tutte le entry scadute
    pub fn cleanup_expired(&self) -> usize {
        let now = Instant::now();
        let mut cache = self.cache.lock().unwrap();
        let expired_keys: Vec<K> = cache
            .iter()
            .filter_map(|(k, (_v, exp))| if *exp <= now { Some(k.clone()) } else { None })
            .collect();
        let deleted = expired_keys.len();
        for k in &expired_keys {
            cache.remove(k);
        }
        if deleted > 0 {
            let mut stats = self.stats.lock().unwrap();
            stats.entries_count -= deleted;
        }
        deleted
    }

    /// Svuota completamente la cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        let mut stats = self.stats.lock().unwrap();
        stats.entries_count = 0;
    }

    /// Restituisce le statistiche correnti
    pub fn get_stats(&self) -> CacheStats {
        let stats = self.stats.lock().unwrap();
        return stats.clone()
    }

    /// Controlla se la cache ha raggiunto la capacità massima
    pub fn is_full(&self) -> bool {
        let cache = self.cache.lock().unwrap();
        if cache.len() >= self.max_capacity {
            return true;
        } else {
            return false;
        }
    }
}

// ------------------ TEST ------------------
#[test]
fn test_cleanup_expired() {
    let cache = CacheManager::new(Duration::from_millis(50), 100);

    // Inserisce entry con TTL brevi
    cache
        .put_with_ttl(
            "key1".to_string(),
            "value1".to_string(),
            Duration::from_millis(30),
        )
        .unwrap();
    cache
        .put_with_ttl(
            "key2".to_string(),
            "value2".to_string(),
            Duration::from_millis(100),
        )
        .unwrap(); // Più lungo

    // Aspetta che key1 scada
    thread::sleep(Duration::from_millis(40));

    // Cleanup manuale
    let cleaned = cache.cleanup_expired();
    assert_eq!(cleaned, 1); // Dovrebbe aver rimosso 1 entry

    // Verifica che key1 sia stata rimossa e key2 sia ancora presente
    assert!(cache.get_cached_only(&"key1".to_string()).is_none());
    assert!(cache.get_cached_only(&"key2".to_string()).is_some());
}

#[test]
fn test_loader_error_handling() {
    let loader: Box<DataLoader<String, String>> = Box::new(|key| {
        if key == "error_key" {
            Err("Database connection failed".to_string())
        } else {
            Ok(format!("loaded_{}", key))
        }
    });

    let cache = CacheManager::with_loader(Duration::from_secs(60), 100, loader);

    // Test caricamento con successo
    let success_result = cache.get(&"good_key".to_string());
    assert!(success_result.is_ok());
    assert!(success_result.unwrap().is_some());

    // Test caricamento con errore
    let error_result = cache.get(&"error_key".to_string());
    assert!(error_result.is_err());
    assert_eq!(error_result.unwrap_err(), "Database connection failed");

    // Verifica che l'errore non abbia corrotto la cache
    let good_again = cache.get(&"good_key".to_string());
    assert!(good_again.is_ok());
    assert!(good_again.unwrap().is_some());
}

#[test]
fn test_clear_cache() {
    let cache = CacheManager::new(Duration::from_secs(60), 100);

    // Inserisce alcune entry
    cache.put("key1".to_string(), "value1".to_string()).unwrap();
    cache.put("key2".to_string(), "value2".to_string()).unwrap();

    assert_eq!(cache.get_stats().entries_count, 2);

    // Svuota la cache
    cache.clear();

    assert_eq!(cache.get_stats().entries_count, 0);
    assert!(cache.get_cached_only(&"key1".to_string()).is_none());
    assert!(cache.get_cached_only(&"key2".to_string()).is_none());
}

#[test]
fn test_is_full() {
    let cache = CacheManager::new(Duration::from_secs(60), 2); // Capacità molto piccola

    assert!(!cache.is_full());

    cache.put("key1".to_string(), "value1".to_string()).unwrap();
    assert!(!cache.is_full());

    cache.put("key2".to_string(), "value2".to_string()).unwrap();
    assert!(cache.is_full());
}
