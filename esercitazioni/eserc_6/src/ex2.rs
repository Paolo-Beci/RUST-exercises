use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    event_tx: Sender<Events>,
    handles: Vec<thread::JoinHandle<()>>,
}

struct Worker {
    id: usize,
    job_rx: Receiver<Job>,
    event_tx: Sender<Events>,
}

enum Events {
    NewJob(Job),
    WorkerDone(usize),
}

impl ThreadPool {
    pub fn new(n: usize) -> Self {
        let (event_tx, event_rx) = channel::<Events>();

        // canali per i worker
        let mut worker_senders = Vec::new();
        let mut handles = Vec::new();

        for id in 0..n {
            let (job_tx, job_rx) = channel::<Job>();
            worker_senders.push(job_tx);

            let event_tx_clone = event_tx.clone();

            // ogni worker gira su un thread
            let handle = thread::spawn(move || {
                let worker = Worker { id, job_rx, event_tx: event_tx_clone };
                worker.run();
            });
            handles.push(handle);
        }

        // scheduler thread
        {
            let worker_senders = worker_senders.clone();
            let event_tx_clone = event_tx.clone();
            thread::spawn(move || {
                let mut queue: Vec<Job> = Vec::new();
                let mut free_workers: Vec<usize> = (0..n).collect();

                while let Ok(event) = event_rx.recv() {
                    match event {
                        Events::NewJob(job) => {
                            if let Some(worker_id) = free_workers.pop() {
                                // assegna subito
                                worker_senders[worker_id].send(job).unwrap();
                            } else {
                                // accoda
                                queue.push(job);
                            }
                        }
                        Events::WorkerDone(id) => {
                            if let Some(job) = queue.pop() {
                                // assegna un job in attesa
                                worker_senders[id].send(job).unwrap();
                            } else {
                                // non ci sono job, segno worker come libero
                                free_workers.push(id);
                            }
                        }
                    }
                }

                drop(event_tx_clone);
            });
        }

        ThreadPool { event_tx, handles }
    }

    pub fn execute(&self, job: Job) {
        self.event_tx.send(Events::NewJob(job)).unwrap();
    }

    pub fn stop(&mut self) {
        for handle in self.handles.drain(..) {
            handle.join().unwrap();
        }
    }
}

impl Worker {
    fn run(self) {
        while let Ok(job) = self.job_rx.recv() {
            // esegui job
            job();

            // notifica fine
            self.event_tx.send(Events::WorkerDone(self.id)).unwrap();
        }
    }
}

// Threadpool
pub fn main_ex2() -> Result<String, Box<dyn std::error::Error>> {
    // alloca i worker
    let threadpool = ThreadPool::new(10);
    for x in 0..100 {
        threadpool.execute(Box::new(move || {
            println!("long running task {}", x);
            thread::sleep(Duration::from_millis(1000))
        }))
    }
    // just to keep the main thread alive
    loop {thread::sleep(Duration::from_millis(1000))};
}