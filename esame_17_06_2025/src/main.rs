// Un applicativo software multithread fa accesso ai servizi di un server remoto, attraverso richieste di tipo HTTP.
// Tali richieste devono includere un token di sicurezza che identifica l'applicativo stesso e ne autorizza l'accesso.
// Per motivi di sicurezza, il token ha una validità limitata nel tempo (qualche minuto) e deve essere rinnovato alla sua scadenza.
// Il token viene ottenuto attraverso una funzione (fornita esternamente e conforme al tipo TokenAcquirer) che restituisce
// alternativamente un token e la sua data di scadenza o un messaggio di errore se non è possibile fornirlo.
// Poiché la emissione richiede un tempo apprezzabile (da alcune centinaia di millisecondi ad alcuni secondi), si vuole
// centralizzare la gestione del token, per evitare che più thread ne facciano richiesta in contemporanea.
// A tale scopo deve essere implementata la struct TokenManager che si occupa di gestire il rilascio, il rinnovo e la messa a
// disposizione del token a chi ne abbia bisogno, secondo la logica di seguito indicata.

// La struct TokenManager offre i seguenti metodi:

// type TokenAcquirer = dyn Fn() => Result<(String, Instant), String> + Sync

// pub fn new(acquire_token: Box<TokenAcquirer> ) -> Self
// pub fn get_token(&self) -> Result<string, string="">
// pub fn try_get_token(&self) -> Option<string>

// Al proprio interno, la struct TokenManager mantiene 3 possibili stati:
// Empty - indica che non è ancora stato richiesto alcun token
// Pending - indica che è in corso una richiesta di acquisizione del token
// Valid - indica che è disponibile un token in corso di validità

// Il metodo new(...) riceve il puntatore alla funzione in grado di acquisire il token. Essa opera in modalità pigra e si
// limita a creare un'istanza della struttura con le necessarie informazioni per gestire il suo successivo comportamento.

// Il metodo get_token(...) implementa il seguente comportamento:
// Se lo stato è Empty, passa allo stato Pending e invoca la funzione per acquisire il token; se questa ritorna un risultato valido,
// memorizza il token e la sua scadenza, porta lo stato a Valid e restituisce copia del token stesso; se, invece, questa restituisce
// un errore, pone lo stato a Empty e restituisce l'errore ricevuto.
// Se lo stato è Pending, attende senza consumare cicli di CPU che questo passi ad un altro valore, dopodiché si comporta di conseguenza.
// Se lo stato è Valid e il token non risulta ancora scaduto, ne restituisce una copia; altrimenti pone lo stato ad Pending e inizia una
// richiesta di acquisizione, come indicato sopra.

// Il metodo try_get_token(...) implementa il seguente comportamento:
// Se lo stato è Valid e il token non è scaduto, restituisce una copia del token opportunamente incapsulata in un oggetto di tipo Option.
// In tutti gli altri casi restituisce None.
// Si implementi tale struttura nel linguaggio Rust.

use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;

#[derive(PartialEq)]
enum State {
    Empty,
    Pending,
    Valid((String, Instant)),
}

fn main() {
    // Entry point required for binary crate.
}

pub struct TokenManager {
    fun: Box<TokenAcquirer>,
    state: Mutex<State>,
    cv: Condvar,
}

type TokenAcquirer = dyn Fn() -> Result<(String, Instant), String> + Send + Sync;

impl TokenManager {
    pub fn new(fun: Box<TokenAcquirer>) -> Self {
        TokenManager {
            fun: fun,
            state: Mutex::new(State::Empty),
            cv: Condvar::new(),
        }
    }

    pub fn get_token(&self) -> Result<String, String> {
        let mut state = self.state.lock().unwrap();
        loop {
            match &*state {
                State::Empty => {
                    *state = State::Pending;
                    drop(state);
                    let res = (self.fun)();
                    state = self.state.lock().unwrap();
                    return match res {
                        Ok((s, i)) => {
                            let r = s.clone();
                            *state = State::Valid((s, i));
                            drop(state);
                            self.cv.notify_all();
                            Ok(r)
                        }
                        Err(s) => {
                            *state = State::Empty;
                            drop(state);
                            self.cv.notify_all();
                            Err(s)
                        }
                    };
                }
                State::Valid((_s, i)) => {
                    let now = Instant::now();
                    if now >= *i {
                        *state = State::Empty;
                        continue;
                    }
                }
                State::Pending => {
                    state = self.cv.wait_while(state, |s| *s == State::Pending).unwrap();
                    continue;
                }
            }
        }
    }

    fn try_get_token(&self) -> Option<String> {
        // Se lo stato è Valid e il token non è scaduto, restituisce una copia del token opportunamente incapsulata in un oggetto di tipo Option.
        // In tutti gli altri casi restituisce None.
        // Si implementi tale struttura nel linguaggio Rust.
        let state = self.state.lock().unwrap();
        match &*state {
            State::Valid((s, i)) if *i > Instant::now() => Some(s.clone()),
            _ => None,
        }
    }
}

// A supporto della validazione del codice realizzato si considerino i seguenti test (due dei quali sono forniti con la relativa
// implementazione, i restanti sono solo indicati e devono essere opportunamente completati):

#[test]
fn a_new_manager_contains_no_token() {
    let a: Box<TokenAcquirer> = Box::new(|| Err("failure".to_string()));
    let manager = TokenManager::new(a);
    assert!(manager.try_get_token().is_none());
}
#[test]
fn a_failing_acquirer_always_returns_an_error() {
    let a: Box<TokenAcquirer> = Box::new(|| Err("failure".to_string()));
    let manager = TokenManager::new(a);
    assert_eq!(manager.get_token(), Err("failure".to_string()));
    assert_eq!(manager.get_token(), Err("failure".to_string()));
}
#[test]
fn a_successful_acquirer_always_returns_success() {
    let a: Box<TokenAcquirer> = Box::new(|| Ok(("abc".to_string(), Instant::now())));
    let manager = TokenManager::new(a);
    assert_eq!(manager.get_token(), Ok("abc".to_string()));
}
#[test]
fn a_slow_acquirer_causes_other_threads_to_wait() {
    use std::time::{Duration, Instant};
    use std::sync::atomic::{AtomicUsize, Ordering};

    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = Arc::clone(&call_count);

    // Token acquirer that simulates a delay
    let a: Box<TokenAcquirer> = Box::new(move || {
        call_count_clone.fetch_add(1, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(500)); // Simulate long acquisition
        Ok(("abc".to_string(), Instant::now() + Duration::from_secs(10)))
    });

    let manager = Arc::new(TokenManager::new(a));

    let manager1 = Arc::clone(&manager);
    let thread1 = thread::spawn(move || {
        assert_eq!(manager1.get_token(), Ok("abc".to_string()));
    });

    // Give thread1 a bit of time to enter `Pending` state
    thread::sleep(Duration::from_millis(100));

    let manager2 = Arc::clone(&manager);
    let thread2 = thread::spawn(move || {
        // This call should wait for thread1 to finish acquiring
        assert_eq!(manager2.get_token(), Ok("abc".to_string()));
    });

    thread1.join().unwrap();
    thread2.join().unwrap();

    // Only one call to the token acquirer should have happened
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
}
