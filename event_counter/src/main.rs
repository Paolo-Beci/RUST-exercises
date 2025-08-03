// ### EventCounter
// Un sistema software monitora in tempo reale una serie di eventi generati da più sensori fisici distribuiti su una rete. 
// Ogni evento è identificato da una categoria (una stringa, es. "temperature", "motion", `"power_loss"`) e ogni sensore, 
// in modo asincrono e indipendente, genera eventi appartenenti a una o più categorie. Il sistema deve fornire una struttura 
// centralizzata e thread-safe per raccogliere e consultare in tempo reale il numero di eventi per ciascuna categoria.
// A tale scopo, si implementi una struttura EventCounter dotata dei seguenti metodi:

use std::sync::Mutex;

fn main() {
    println!("Hello, world!");
}

pub struct EventCounter { 
    category_counter: Mutex<Vec<(String, usize)>>,
}

impl EventCounter {
    pub fn new() -> Self {
        EventCounter { category_counter: Mutex::new(Vec::new()) }
    }

    /// Registra un nuovo evento per la categoria specificata.
    /// Se la categoria non è ancora presente, viene creata.
    pub fn record_event(&self, category: &str) {
        let mut collection = self.category_counter.lock().unwrap();
        if let Some((_, count)) = collection.iter_mut().find(|(cat, _)| cat == category) {
            *count += 1;
        } else {
            collection.push((category.to_string(), 1));
        }
    }

    /// Restituisce il numero di eventi registrati per una data categoria.
    /// Se la categoria non è mai stata vista, restituisce 0.
    pub fn get_count(&self, category: &str) -> usize {
        let collection = self.category_counter.lock().unwrap();
        if let Some((_, count)) = collection.iter().find(|(cat, _)| cat == category) {
            return *count
        } else {
            return 0
        }
    }

    /// Restituisce una lista di tutte le categorie e i relativi conteggi.
    /// L'ordine non è rilevante.
    pub fn snapshot(&self) -> Vec<(String, usize)> {
        let collection = self.category_counter.lock().unwrap();
        return collection.clone()
    }
}


// -------------------- TEST ----------------------
#[test]
fn new_counter_has_zero_for_all() {
    let counter = EventCounter::new();
    assert_eq!(counter.get_count("motion"), 0);
    assert_eq!(counter.get_count("temperature"), 0);
}

#[test]
fn record_event_increases_count() {
    let counter = EventCounter::new();
    counter.record_event("motion");
    counter.record_event("motion");
    assert_eq!(counter.get_count("motion"), 2);
}

#[test]
fn snapshot_returns_all_counts() {
    let counter = EventCounter::new();
    counter.record_event("a");
    counter.record_event("b");
    counter.record_event("a");

    let mut snapshot = counter.snapshot();
    snapshot.sort(); // ordine non garantito

    assert_eq!(snapshot, vec![("a".to_string(), 2), ("b".to_string(), 1)]);
}

#[test]
fn concurrent_recording_is_safe() {
    use std::sync::Arc;
    use std::thread;

    let counter = Arc::new(EventCounter::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                c.record_event("event");
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(counter.get_count("event"), 10_000);
}