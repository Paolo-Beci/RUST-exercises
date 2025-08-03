// Un sistema di monitoraggio all'interno di uno stabilimento industriale raccoglie misure di temperatura da più
// sensori. Le misure vengono raccolte in modo asincrono, sono automaticamente etichettate con
// l'istante temporale in cui sono comunicate e possono essere inviate da più thread
// contemporaneamente. Compito del sistema è quello di aggregare le misure ricevute, calcolando la
// temperatura media e il numero di misurazioni ricevute da ciascun sensore, operando un campionamento ad
// intervalli regolari indicati dal parametro passato alla funzione di costruzione. In tale periodo, un sensore può
// inviare più misure, che devono essere tutte considerate nel calcolo della media. Un thread interno alla
// struttura si occupa di calcolare la media delle temperature per ciascun sensore, aggiornandola secondo il
// periodo di campionamento indicato. All'atto della distruzione della struttura, il thread interno deve essere
// terminato in modo sicuro. Per implementare tale sistema, si richiede di realizzare la struct Aggregator che
// oﬀre i seguenti metodi thread-safe:

use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

struct Measurement {
    id: usize,
    timestamp: Instant,
    measure: f64,
}

struct InnerState {
    running: bool,
    measurements: Vec<Measurement>,
    sample_time: Instant,
    recent_averages: Vec<Average>,
}

pub struct Aggregator {
    // campi privati
    state: Arc<(Mutex<InnerState>, Condvar)>,
    join_handle: Option<JoinHandle<()>>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Average {
    pub sensor_id: usize,
    pub reference_time: Instant, //indica l'istante temporale in cui è stata calcolata la media
    pub average_temperature: f64, 
}

impl Aggregator {
    pub fn new(sample_time_millis: u64) -> Self {
        // implementazione del costruttore
        let state = Arc::new((
            Mutex::new(InnerState {
                running: true,
                measurements: vec![],
                sample_time: Instant::now(),
                recent_averages: vec![],
            }),
            Condvar::new(),
        ));

        let thread_state = state.clone();

        let join_handle = std::thread::spawn(move || {
            let (mutex, condvar) = &*thread_state;

            let mut inner_state = mutex.lock().unwrap();

            loop {
                let next_wakeup = inner_state.sample_time + Duration::from_millis(sample_time_millis);
                let sleep_time = next_wakeup.saturating_duration_since(Instant::now());

                let result = condvar
                    .wait_timeout_while(inner_state, sleep_time, |s| s.running)
                    .unwrap();

                inner_state = result.0;

                if !inner_state.running {
                    break;
                }

                inner_state.sample_time = next_wakeup;

                // Extract measurements up to the current sample time
                let mut measurements: Vec<Measurement> = Vec::new();
                inner_state.measurements.retain(|m| {
                    if m.timestamp < next_wakeup {
                        measurements.push(Measurement {
                            id: m.id,
                            timestamp: m.timestamp,
                            measure: m.measure,
                        });
                        false // remove from measurements
                    } else {
                        true // keep in measurements
                    }
                });

                drop(inner_state); // Release the lock during computation

                // Compute averages
                let mut averages = HashMap::<usize, (f64, usize)>::new();

                for m in &measurements {
                    averages
                        .entry(m.id)
                        .and_modify(|(sum, count)| {
                            *sum += m.measure;
                            *count += 1;
                        })
                        .or_insert((m.measure, 1));
                }

                let new_averages: Vec<Average> = averages
                    .into_iter()
                    .map(|(id, (measure, count))| Average {
                        sensor_id: id,
                        reference_time: next_wakeup,
                        average_temperature: measure / count as f64,
                    })
                    .collect();

                // Store the result
                inner_state = mutex.lock().unwrap();
                inner_state.recent_averages = new_averages;
            }
        });

        Self {
            state,
            join_handle: Some(join_handle),
        }
    }

    pub fn add_measure(&self, sensor_id: usize, temperature: f64) {
        // aggiunge una misura di temperatura per il sensore con id `sensor_id` e temperatura `temperature`.
        // Le misure sono automaticamente etichettate
        // con l'istante temporale in cui sono comunicate.
        let now = Instant::now();
        let mut state = self.state.0.lock().unwrap();

        state.measurements.push(Measurement {
            id: sensor_id,
            timestamp: now,
            measure: temperature,
        });
    }

    pub fn get_averages(&self) -> Vec<Average> {
        // restituisce un vettore che riporta la temperatura media di ciascun sensore,
        // calcolata durante l'ultimo periodo di campionamento.
        // Sono presenti solo i sensori che hanno inviato almeno una misura.
        let state = self.state.0.lock().unwrap();
        state.recent_averages.clone()
    }
}

impl Drop for Aggregator {
    fn drop(&mut self) {
        // Signal the background thread to stop
        let mut state = self.state.0.lock().unwrap();
        state.running = false;
        drop(state);

        // Notify the background thread in case it's sleeping
        self.state.1.notify_all();

        // Join the background thread to ensure clean shutdown
        if let Some(join_handle) = self.join_handle.take() {
            join_handle.join().unwrap();
        }
    }
}

fn main() {
    println!("Hello, world!");
}



#[cfg(test)]
mod tests {
    use super::{Aggregator, Average};
    use std::time::Duration;

    #[test]
    fn when_no_measures_are_sent_an_empty_state_is_returned() {
        let aggregator = Aggregator::new(10);
        let averages = aggregator.get_averages();
        assert!(averages.is_empty());
    }

    #[test]
    fn when_a_single_measure_is_sent_it_is_returned() {
        let aggregator = Aggregator::new(20);
        std::thread::sleep(std::time::Duration::from_millis(1));
        aggregator.add_measure(1, 1.0);
        assert!(aggregator.get_averages().is_empty());
        std::thread::sleep(Duration::from_millis(25));
        let averages = aggregator.get_averages();
        assert_eq!(averages.len(),  1);
        assert!(matches!(averages.get(0), Some(&Average{ sensor_id:1, average_temperature:1.0, .. })));
    }
    #[test]
    fn when_two_measures_are_sent_their_average_is_returned() {
        let aggregator = Aggregator::new(100);
        aggregator.add_measure(1, 1.0);
        aggregator.add_measure(1, 2.0);
        std::thread::sleep(Duration::from_millis(110));
        let averages = aggregator.get_averages();
        assert_eq!(averages.len(),  1);
        assert!(matches!(averages.get(0), Some(&Average{ sensor_id:1, average_temperature:1.5, .. })));
    }
    #[test]
    fn when_two_measures_are_sent_from_different_sensors_their_average_is_returned() {
        let aggregator = Aggregator::new(100);
        aggregator.add_measure(1, 1.0);
        aggregator.add_measure(2, 2.0);
        aggregator.add_measure(2, 1.0);
        aggregator.add_measure(1, 2.0);
        std::thread::sleep(Duration::from_millis(110));
        let averages = aggregator.get_averages();
        assert_eq!(averages.len(),  2);
        let timestamp = averages.get(0).unwrap().reference_time;
        assert!(averages.contains(&Average{ sensor_id:1, average_temperature:1.5, reference_time: timestamp }));
        assert!(averages.contains(&Average{ sensor_id:2, average_temperature:1.5, reference_time: timestamp }));
    }

    #[test]
    fn more_threads_may_send_data() {
        let aggregator = Aggregator::new(100);
        std::thread::scope(|s| {
            s.spawn(|| {
                aggregator.add_measure(1, 1.0);
                std::thread::sleep(Duration::from_millis(5));
                aggregator.add_measure(1, 3.0);
            });
            s.spawn(|| {
                aggregator.add_measure(2, 2.0);
                std::thread::sleep(Duration::from_millis(5));
                aggregator.add_measure(2, 8.0);
            });
        });
        std::thread::sleep(Duration::from_millis(110));
        let averages = aggregator.get_averages();
        assert_eq!(averages.len(),  2);
        let timestamp = averages.get(0).unwrap().reference_time;
        assert!(averages.contains(&Average{ sensor_id:1, average_temperature:2.0, reference_time: timestamp }));
        assert!(averages.contains(&Average{ sensor_id:2, average_temperature:5.0, reference_time: timestamp }));
    }
    #[test]
    fn an_aggregator_shuts_down_cleanly() {
        {
            let _aggregator = Aggregator::new(10);
        }
        assert!(true);
    }
}