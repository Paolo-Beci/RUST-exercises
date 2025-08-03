// ### LazyCache
// Un sistema distribuito interroga dati remoti (come configurazioni o metadati) tramite richieste costose in termini di tempo. 
// Per ottimizzare le prestazioni, si desidera implementare un sistema di caching centralizzato, thread-safe, con inizializzazione 
// lazy per chiave, evitando che più thread interroghino il server per la stessa chiave contemporaneamente.

use std::{collections::HashMap, sync::Mutex};

fn main() {
    println!("Hello, world!");
}

type FetchFn = dyn Fn(&str) -> Result<String, String> + Sync + Send;

pub struct LazyCache { 
    cache: Mutex<HashMap<String, String>>,
    fetcher: Box<FetchFn>
}

impl LazyCache {
    pub fn new(fetcher: Box<FetchFn>) -> Self {
        return LazyCache {cache: Mutex::new(HashMap::new()), fetcher: fetcher}
    }

    /// Restituisce il valore associato alla chiave, eseguendo la fetch se necessario.
    /// Se un altro thread sta già caricando quella chiave, attende il risultato.
    pub fn get(&self, key: &str) -> Result<String, String> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(val) = cache.get(key) {
                return Ok(val.clone());  // key già esiste
            }
        }
        let result = (self.fetcher)(key);
        if let Ok(ref v) = result {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(key.to_string(), v.clone());
        }
        return result;
    }
}

// ---------------------- TEST --------------------
#[test]
fn initial_get_triggers_fetch() {
    let f: Box<FetchFn> = Box::new(|k| Ok(format!("val:{}", k)));
    let cache = LazyCache::new(f);
    assert_eq!(cache.get("a"), Ok("val:a".to_string()));
}

#[test]
fn repeated_get_does_not_trigger_fetch_again() {
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let c = counter.clone();
    let f: Box<FetchFn> = Box::new(move |k| {
        c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(format!("v:{}", k))
    });

    let cache = LazyCache::new(f);
    assert_eq!(cache.get("x"), Ok("v:x".to_string()));
    assert_eq!(cache.get("x"), Ok("v:x".to_string()));
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn fetch_failure_is_not_cached() {
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let c = counter.clone();
    let f: Box<FetchFn> = Box::new(move |_| {
        c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Err("fail".to_string())
    });

    let cache = LazyCache::new(f);
    assert_eq!(cache.get("k"), Err("fail".to_string()));
    assert_eq!(cache.get("k"), Err("fail".to_string()));
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
}

#[test]
fn concurrent_gets_only_trigger_one_fetch() {
    use std::thread;
    use std::sync::{Arc, Barrier, atomic::{AtomicUsize, Ordering}};

    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    let f: Box<FetchFn> = Box::new(move |_| {
        c.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(std::time::Duration::from_millis(100));
        Ok("ready".to_string())
    });

    let cache = Arc::new(LazyCache::new(f));
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    for _ in 0..10 {
        let cache = cache.clone();
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            barrier.wait();
            assert_eq!(cache.get("shared"), Ok("ready".to_string()));
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}