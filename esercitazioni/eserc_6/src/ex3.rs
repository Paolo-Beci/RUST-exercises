use std::thread;
use reqwest::blocking;
use std::time::Duration;

struct Downloader {
    source: String,
    timeout: u64
}

impl Downloader {
    fn new(source: &str, timeout: u64) -> Self {
        Downloader { source: source.to_string(), timeout: timeout }
    }

    fn start(&self) -> Result<String, Box<dyn std::error::Error + Send>> {
        let (tx, rx) = std::sync::mpsc::channel();
        let url = self.source.clone();

        thread::spawn(move || {
            let result = (|| {
                // thread::sleep(Duration::from_secs(10));  // TEST
                let resp = blocking::get(&url).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
                if resp.status().is_success() {
                    let text = resp.text().map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
                    Ok(text)
                } else {
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Request failed with status: {}", resp.status()),
                    )) as Box<dyn std::error::Error + Send>)
                }
            })();

            // Manda il risultato al main thread
            let _ = tx.send(result);
        });

        // Timeout gestito fuori dal thread
        match rx.recv_timeout(Duration::from_secs(self.timeout)) {
            Ok(res) => res,
            Err(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Request timed out",
            ))),
        }
    }
}

// Processi
pub fn main_ex3() -> Result<String, Box<dyn std::error::Error + Send>> {
    let downloader = Downloader::new("http://www.google.com", 10);
    match downloader.start() {
        Ok(data) => {println!("Data: {}", data)},
        Err(e) => {println!("Error: {}", e)}
    }

    Ok("OK".to_string())
}