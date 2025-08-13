use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct CyclicBarrier {
    state: Arc<(Mutex<BarrierState>, Condvar)>,
    parties: usize, // numero totale di thread che devono aspettare
}

struct BarrierState {
    count: usize, // thread mancanti
    generation: usize, // numero di barriere superate
}

impl Clone for CyclicBarrier {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            parties: self.parties,
        }
    }
}

impl CyclicBarrier {
    fn new(n: usize) -> Self {
        Self {
            state: Arc::new((
                Mutex::new(BarrierState { count: n, generation: 0 }),
                Condvar::new(),
            )),
            parties: n,
        }
    }

    fn wait(&self) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap();
        let gen = state.generation;

        state.count -= 1;

        if state.count == 0 {
            // reset
            state.count = self.parties;
            state.generation += 1;
            cvar.notify_all();
        } else {
            // aspetta fino alla prossima barriera
            state = cvar
                .wait_while(state, |s| s.generation == gen)
                .unwrap();
        }
    }
}

pub fn main_ex3() -> Result<String, Box<dyn std::error::Error>> {
    let barrier = CyclicBarrier::new(5);
    let mut vt = Vec::new();

    for i in 0..5 {
        let b = barrier.clone();
        vt.push(thread::spawn(move || {
            for j in 0..3 {
                println!("Thread {} before barrier {}", i, j);
                b.wait();
                println!("Thread {} after  barrier {}", i, j);
            }
        }));
    }

    for t in vt {
        t.join().unwrap();
    }

    Ok("OK".to_string())
}
