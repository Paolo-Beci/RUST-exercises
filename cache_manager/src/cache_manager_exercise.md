# CacheManager Distribuito

## Scenario
Un'applicazione web ad alta disponibilità deve gestire una cache distribuita per ottimizzare le performance delle richieste verso un database. La cache deve supportare operazioni concorrenti da parte di più thread, implementare una strategia di invalidazione intelligente e garantire consistenza dei dati anche in presenza di aggiornamenti simultanei.

## Obiettivo
Implementare una struttura `CacheManager<K, V>` in Rust che gestisca una cache thread-safe con le seguenti caratteristiche:

### Funzionalità Richieste

1. **Cache con TTL (Time To Live)**: Ogni entry ha una scadenza configurabile
2. **Invalidazione Automatica**: Rimozione automatica delle entry scadute
3. **Cache-aside Pattern**: Possibilità di caricare dati dal backend se non presenti in cache
4. **Statistiche Real-time**: Contatori per hit/miss e operazioni

### Interfaccia della Struttura

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::hash::Hash;

// Tipo per la funzione di caricamento dal backend
type DataLoader<K, V> = dyn Fn(&K) -> Result<V, String> + Send + Sync;

pub struct CacheManager<K, V> {
    // Implementazione richiesta
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub entries_count: usize,
}

impl<K, V> CacheManager<K, V> 
where 
    K: Clone + Hash + Eq + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Crea un nuovo CacheManager con TTL di default e capacità massima
    pub fn new(default_ttl: Duration, max_capacity: usize) -> Self {
        // TODO: implementare
    }

    /// Crea un nuovo CacheManager con funzione di caricamento dal backend
    pub fn with_loader(
        default_ttl: Duration, 
        max_capacity: usize,
        loader: Box<DataLoader<K, V>>
    ) -> Self {
        // TODO: implementare
    }

    /// Inserisce un valore nella cache con TTL di default
    pub fn put(&self, key: K, value: V) -> Result<(), String> {
        // TODO: implementare
    }

    /// Inserisce un valore nella cache con TTL personalizzato
    pub fn put_with_ttl(&self, key: K, value: V, ttl: Duration) -> Result<(), String> {
        // TODO: implementare
    }

    /// Recupera un valore dalla cache
    /// Se non presente e il loader è configurato, tenta di caricarlo dal backend
    pub fn get(&self, key: &K) -> Result<Option<V>, String> {
        // TODO: implementare
    }

    /// Recupera un valore dalla cache senza utilizzare il loader
    pub fn get_cached_only(&self, key: &K) -> Option<V> {
        // TODO: implementare
    }

    /// Rimuove un valore dalla cache
    pub fn remove(&self, key: &K) -> bool {
        // TODO: implementare
    }

    /// Invalida tutte le entry scadute
    pub fn cleanup_expired(&self) -> usize {
        // TODO: implementare
    }

    /// Svuota completamente la cache
    pub fn clear(&self) {
        // TODO: implementare
    }

    /// Restituisce le statistiche correnti
    pub fn get_stats(&self) -> CacheStats {
        // TODO: implementare
    }

    /// Controlla se la cache ha raggiunto la capacità massima
    pub fn is_full(&self) -> bool {
        // TODO: implementare
    }
}
```

## Requisiti di Implementazione

### 1. Thread Safety
- Tutte le operazioni devono essere thread-safe
- Utilizzare le primitive di sincronizzazione appropriate (`Mutex`, `RwLock`, `Arc`)
- Evitare deadlock e race conditions

### 2. Gestione della Memoria
- Implementare una strategia LRU (Least Recently Used) per l'eviction quando si raggiunge la capacità massima
- Cleanup automatico delle entry scadute durante le operazioni

### 3. Performance
- Le operazioni di lettura (`get`, `get_cached_only`) dovrebbero essere ottimizzate per la concorrenza
- Minimizzare il tempo di lock delle strutture condivise

### 4. Gestione degli Errori
- Gestire appropriatamente i casi di errore del data loader
- Fornire messaggi di errore informativi

## Test di Validazione

Implementare i seguenti test per validare il comportamento:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_put_and_get() {
        // TODO: Test base per inserimento e recupero
    }

    #[test]
    fn test_ttl_expiration() {
        // TODO: Test per verifica scadenza TTL
    }

    #[test]
    fn test_concurrent_access() {
        // TODO: Test per accesso concorrente da più thread
    }

    #[test]
    fn test_cache_with_loader() {
        // TODO: Test per funzionamento con data loader
    }

    #[test]
    fn test_lru_eviction() {
        // TODO: Test per verifica eviction LRU
    }

    #[test]
    fn test_stats_tracking() {
        // TODO: Test per verifica statistiche
    }

    #[test]
    fn test_cleanup_expired() {
        // TODO: Test per cleanup manuale delle entry scadute
    }

    #[test]
    fn test_loader_error_handling() {
        // TODO: Test per gestione errori del loader
    }
}
```

## Suggerimenti per l'Implementazione

1. **Struttura Interna**: Considera l'uso di una `HashMap` per lo storage principale e una struttura ausiliaria per tracciare l'ordine di accesso (LRU)

2. **Sincronizzazione**: Valuta l'uso di `RwLock` per permettere letture concorrenti e `Mutex` per le statistiche

3. **Entry Structure**: Definisci una struttura interna per le entry che includa valore, timestamp di creazione e ultimo accesso

4. **Background Cleanup**: Considera l'implementazione di un cleanup periodico in background (opzionale, avanzato)

## Criteri di Valutazione

- **Correttezza**: Tutti i test devono passare
- **Thread Safety**: Nessun data race o deadlock
- **Performance**: Operazioni efficienti anche sotto carico concorrente
- **Qualità del Codice**: Codice pulito, ben documentato e idiomatico Rust
- **Gestione degli Errori**: Handling appropriato di tutti i casi edge

## Bonus (Opzionale)

- Implementare metriche aggiuntive (latenza media, throughput)
- Aggiungere supporto per serializzazione/deserializzazione delle entry
- Implementare un meccanismo di warming della cache all'avvio
- Aggiungere logging per debugging e monitoring