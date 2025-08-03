// Devi implementare una struct PermitManager che gestisce un numero limitato di permessi simultanei utilizzabili 
// da più thread in parallelo. Essa modella una risorsa condivisa a capacità limitata (come un semaforo) e permette di:
// Richiedere un permesso (eventualmente aspettando se non ce ne sono disponibili)
// Rilasciare un permesso
// Tentarne l'acquisizione in modo non bloccante o con timeout

use std::{sync::{Arc, Condvar, Mutex}, thread, time::{Duration, Instant}};

fn main() {
    println!("Hello, world!");
}

struct PermitManager {
    permits: Mutex<usize>,
    cv: Condvar
}

impl PermitManager {
    pub fn new(max_permits: usize) -> Self {
        // inizializza la struttura con un numero massimo di permessi disponibili
        return PermitManager { permits: Mutex::new(max_permits), cv: Condvar::new() }
    }

    pub fn acquire(&self) {
        // blocca finché un permesso non è disponibile, e poi lo acquisisce
        let mut permits = self.permits.lock().unwrap();
        loop {
            if *permits == 0 {
                self.cv.wait(permits);
                return;
            } else {
                *permits -= 1;
                return;
            }
        }
    }

    pub fn try_acquire(&self) -> bool {
        // tenta di acquisire un permesso: ritorna true se ci riesce, false altrimenti
        let mut permits = self.permits.lock().unwrap();
        if *permits == 0 {
            return false;
        } else {
            *permits -= 1;
            return true;
        }
    }

    pub fn acquire_timeout(&self, dur: Duration) -> bool {
        // prova ad acquisire un permesso aspettando al massimo dur. Se riesce in tempo ritorna true, altrimenti false
        let permits = self.permits.lock().unwrap();
        let (mut permits, result) = self.cv.wait_timeout_while(permits, dur, |p| {*p==0}).unwrap();
        if result.timed_out() || *permits == 0 {
            false
        } else {
            *permits -= 1;
            true
        }
    }

    pub fn release(&self) {
        // rilascia un permesso precedentemente acquisito
        let mut permits = self.permits.lock().unwrap();
        *permits += 1;
    }
}


// -------------------------- TESTS ------------------------------------
#[test]
fn new_manager_allows_max_permits() {
    let manager = PermitManager::new(3);
    assert!(manager.try_acquire());
    assert!(manager.try_acquire());
    assert!(manager.try_acquire());
    assert!(!manager.try_acquire()); // Esauriti
}

#[test]
fn acquire_blocks_until_permit_is_available() {
    let manager = Arc::new(PermitManager::new(1));
    assert!(manager.try_acquire());

    let m_clone = Arc::clone(&manager);
    let handle = thread::spawn(move || {
        m_clone.acquire(); // deve aspettare
        m_clone.release();
    });

    thread::sleep(Duration::from_millis(100));
    manager.release(); // sblocca il thread

    handle.join().unwrap();
}

#[test]
fn acquire_timeout_works_correctly() {
    let manager = PermitManager::new(1);
    assert!(manager.try_acquire());
    let start = Instant::now();
    let acquired = manager.acquire_timeout(Duration::from_millis(200));
    let elapsed = start.elapsed();
    assert!(!acquired);
    assert!(elapsed >= Duration::from_millis(200));
}

#[test]
fn permits_are_reusable() {
    let manager = PermitManager::new(2);
    assert!(manager.try_acquire());
    assert!(manager.try_acquire());
    assert!(!manager.try_acquire());
    manager.release();
    assert!(manager.try_acquire());
}
