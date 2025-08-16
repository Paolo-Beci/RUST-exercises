use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

pub struct CyclicBarrier {
    n: usize,
    senders: Vec<Sender<()>>,
    receivers: Vec<Option<Receiver<()>>>, // ogni receiver verrà "consumato" con take()
}

pub struct Waiter {
    my_receiver: Receiver<()>,
    my_senders: Vec<Sender<()>>,
}

impl CyclicBarrier {
    pub fn new(n: usize) -> Self {
        assert!(n > 0, "CyclicBarrier size must be > 0");

        let mut senders = Vec::with_capacity(n);
        let mut receivers: Vec<Option<Receiver<()>>> = Vec::with_capacity(n);

        // Crea n canali indipendenti (ognuno ha un receiver dedicato a un thread)
        for _ in 0..n {
            let (tx, rx) = channel();
            senders.push(tx);
            receivers.push(Some(rx)); 
        }

        CyclicBarrier { n, senders, receivers }
    }

    // Restituisce il Waiter per l'indice `id` spostando il suo Receiver
    pub fn get_waiter(&mut self, id: usize) -> Waiter {
        assert!(id < self.n, "waiter id out of range");

        // Sposta (move) il receiver fuori dal vettore; fallisce se già preso
        let my_receiver = self.receivers[id]
            .take()
            .expect("Waiter already taken for this id");

        // Colleziona tutti i sender verso gli ALTRI thread (n-1)
        let mut my_senders = Vec::with_capacity(self.n - 1);
        for (j, s) in self.senders.iter().enumerate() {
            if j != id {
                my_senders.push(s.clone());
            }
        }

        Waiter { my_receiver, my_senders }
    }
}

impl Waiter {
    pub fn wait(&self) {
        // 1) segnala a tutti gli altri thread
        for s in &self.my_senders {
            // Se un thread è morto, send può fallire: qui ignoriamo l'errore e lasciamo che recv blocchi;
            // in una versione robusta potresti gestire l'errore e abortire.
            let _ = s.send(());
        }

        // 2) attende n-1 segnali sul proprio receiver
        for _ in 0..self.my_senders.len() {
            // Se un mittente è chiuso e non arriveranno abbastanza messaggi, qui si bloccherebbe per sempre.
            // È il comportamento atteso di una barriera: se qualcuno non arriva, gli altri restano in attesa.
            let _ = self.my_receiver.recv();
        }
    }
}

// Barriera ciclica con canali
pub fn main_ex1() -> Result<String, Box<dyn std::error::Error>> {
    let mut cbarrier = CyclicBarrier::new(3);
    let mut vt = Vec::new();

    for i in 0..3 {
        let waiter = cbarrier.get_waiter(i);
        vt.push(thread::spawn(move || {
            for j in 0..10 {
                waiter.wait();
                println!("after barrier {} {}", i, j);
            }
        }));
    }

    // aspetta la fine dei thread (altrimenti il main termina prima)
    for h in vt {
        h.join().expect("thread panicked");
    }

    Ok("OK".to_string())
}
