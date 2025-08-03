// Un CancelableLatch è un tratto di sincronizzazione che permette a uno o più thread di attendere, senza
// consumare cicli di CPU, che altri thread eseguano i propri compiti e ne segnalino l'esito.
// All'atto della creazione occorre indicare il numero di compiti da attendere.
// Il tratto oﬀre il metodo count_down() che permette di indicare che uno dei compiti è terminato con successo:
// se non restano altri compiti da attendere, le attese vengono sbloccate con successo, altrimenti proseguono.
// Il metodo cancel() permette di segnalare che uno dei compiti è fallito: in questo caso, le attese vengono
// subito sbloccate indicando l'avvenuta cancellazione.
// Il tratto oﬀre due metodi di attesa: uno incondizionato (ovvero, l'attesa si protrae fino a che tutti i compiti
// sono stati terminati con successo o è stata richiesta una cancellazione) e uno con timeout (in questo caso,
// l'attesa può terminare anche se entro il tempo indicato non si raggiungono le condizioni precedenti: in tale
// caso viene segnalato che il tempo è scaduto).
// Si realizzi, usando il linguaggio Rust, una struttura che implementi tale tratto.

use std::{sync::{Arc, Condvar, Mutex}, time::Duration};

#[derive(PartialEq, Eq, Debug)]
pub enum WaitResult {
    Success,
    Timeout,
    Canceled
}

pub trait CancelableLatch {
    fn new(count: usize) -> Self;
    fn count_down(&self);
    fn cancel(&self);
    fn wait(&self) -> WaitResult;
    fn wait_timeout(&self, d: Duration) -> WaitResult;
}

struct Counter {
    count: Arc<Mutex<(usize, bool)>>,
    cv: Condvar
}

impl CancelableLatch for Counter {
    fn new(count: usize) -> Self {
        return Counter {count: Arc::new(Mutex::new((count, false))), cv: Condvar::new()}
    }

    fn count_down(&self) {
        let mut guard = self.count.lock().unwrap();
        let (count, _canceled) = &mut *guard;
        if *count > 0 {
            *count -= 1;
            if *count == 0 {
                self.cv.notify_all();
            }
        } else {
            self.cv.notify_all();
        }
    }

    fn cancel(&self) {
        let mut guard = self.count.lock().unwrap();
        let (_count, canceled) = &mut *guard;
        *canceled = true;
        self.cv.notify_all();
    }

    fn wait(&self) -> WaitResult {
        let mut guard = self.count.lock().unwrap();
        while guard.0 > 0 && !guard.1 {
            guard = self.cv.wait(guard).unwrap();
        }
        if guard.1 {
            WaitResult::Canceled
        } else {
            WaitResult::Success
        }
    }

    fn wait_timeout(&self, d: Duration) -> WaitResult {
        let guard = self.count.lock().unwrap();
        let result = self.cv.wait_timeout_while(guard, d, |(count, canceled)| {
            *count > 0 && !*canceled
        }).unwrap();
        let (count, canceled) = &*result.0;
        if *canceled {
            WaitResult::Canceled
        } else if *count == 0 {
            WaitResult::Success
        } else if result.1.timed_out() {
            WaitResult::Timeout
        } else {
            WaitResult::Timeout // fallback, should not happen
        }
    }
}

fn main() {
    // Entry point required for binary crate.
}


// ------------------------- TESTS ------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn test_count_down_to_zero() {
        let latch = Counter::new(2);
        
        // Count down twice
        latch.count_down();
        latch.count_down();
        
        // Should succeed immediately since count is 0
        let result = latch.wait();
        assert_eq!(result, WaitResult::Success);
    }

    #[test]
    fn test_wait_with_timeout_success() {
        let latch = Counter::new(1);
        latch.count_down(); // Count down immediately
        
        let result = latch.wait_timeout(Duration::from_millis(100));
        assert_eq!(result, WaitResult::Success);
    }

    #[test]
    fn test_wait_with_timeout_expires() {
        let latch = Counter::new(1);
        
        // Wait with a short timeout, should timeout
        let start = Instant::now();
        let result = latch.wait_timeout(Duration::from_millis(50));
        let elapsed = start.elapsed();
        
        assert_eq!(result, WaitResult::Timeout);
        assert!(elapsed >= Duration::from_millis(45)); // Allow some tolerance
    }

    #[test]
    fn test_cancel_before_wait() {
        let latch = Counter::new(2);
        
        // Cancel before waiting
        latch.cancel();
        
        let result = latch.wait();
        assert_eq!(result, WaitResult::Canceled);
    }

    #[test]
    fn test_cancel_during_wait() {
        let latch = Arc::new(Counter::new(2));
        let latch_clone = latch.clone();
        
        // Spawn a thread that cancels after a short delay
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            latch_clone.cancel();
        });
        
        let result = latch.wait();
        assert_eq!(result, WaitResult::Canceled);
    }

    #[test]
    fn test_multiple_waiters_success() {
        let latch = Arc::new(Counter::new(2));
        let mut handles = vec![];
        
        // Spawn multiple waiting threads
        for _ in 0..3 {
            let latch_clone = latch.clone();
            let handle = thread::spawn(move || {
                latch_clone.wait()
            });
            handles.push(handle);
        }
        
        // Count down to zero
        thread::sleep(Duration::from_millis(10));
        latch.count_down();
        latch.count_down();
        
        // All waiters should succeed
        for handle in handles {
            let result = handle.join().unwrap();
            assert_eq!(result, WaitResult::Success);
        }
    }

    #[test]
    fn test_multiple_waiters_cancel() {
        let latch = Arc::new(Counter::new(2));
        let mut handles = vec![];
        
        // Spawn multiple waiting threads
        for _ in 0..3 {
            let latch_clone = latch.clone();
            let handle = thread::spawn(move || {
                latch_clone.wait()
            });
            handles.push(handle);
        }
        
        // Cancel after a short delay
        thread::sleep(Duration::from_millis(10));
        latch.cancel();
        
        // All waiters should be canceled
        for handle in handles {
            let result = handle.join().unwrap();
            assert_eq!(result, WaitResult::Canceled);
        }
    }

    #[test]
    fn test_count_down_more_than_initial() {
        let latch = Counter::new(2);
        
        // Count down more times than initial count
        latch.count_down();
        latch.count_down();
        latch.count_down(); // Extra count down
        
        let result = latch.wait();
        assert_eq!(result, WaitResult::Success);
    }

    #[test]
    fn test_zero_initial_count() {
        let latch = Counter::new(0);
        
        // Should succeed immediately
        let result = latch.wait();
        assert_eq!(result, WaitResult::Success);
    }

    #[test]
    fn test_timeout_with_partial_countdown() {
        let latch = Counter::new(2);
        
        // Count down only once
        latch.count_down();
        
        // Should timeout since count is still 1
        let start = Instant::now();
        let result = latch.wait_timeout(Duration::from_millis(50));
        let elapsed = start.elapsed();
        
        assert_eq!(result, WaitResult::Timeout);
        assert!(elapsed >= Duration::from_millis(45));
    }

    #[test]
    fn test_concurrent_count_down() {
        let latch = Arc::new(Counter::new(4));
        let mut handles = vec![];
        
        // Spawn multiple threads that count down
        for _ in 0..4 {
            let latch_clone = latch.clone();
            let handle = thread::spawn(move || {
                thread::sleep(Duration::from_millis(10));
                latch_clone.count_down();
            });
            handles.push(handle);
        }
        
        // Wait for completion
        let result = latch.wait();
        assert_eq!(result, WaitResult::Success);
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_timeout_vs_cancel_race() {
        let latch = Arc::new(Counter::new(1));
        let latch_clone = latch.clone();
        
        // Spawn a thread that cancels after a delay
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(25));
            latch_clone.cancel();
        });
        
        // Wait with timeout that should expire around the same time
        let result = latch.wait_timeout(Duration::from_millis(30));
        
        // Should be either Canceled or Timeout, but not Success
        assert!(result == WaitResult::Canceled || result == WaitResult::Timeout);
    }

    #[test]
    fn test_wait_after_cancel() {
        let latch = Counter::new(2);
        
        // Cancel first
        latch.cancel();
        
        // Multiple waits should all return Canceled
        assert_eq!(latch.wait(), WaitResult::Canceled);
        assert_eq!(latch.wait(), WaitResult::Canceled);
        assert_eq!(latch.wait_timeout(Duration::from_millis(10)), WaitResult::Canceled);
    }

    #[test]
    fn test_count_down_after_cancel() {
        let latch = Counter::new(2);
        
        // Cancel first
        latch.cancel();
        
        // Count down should not change the canceled state
        latch.count_down();
        latch.count_down();
        
        let result = latch.wait();
        assert_eq!(result, WaitResult::Canceled);
    }
}