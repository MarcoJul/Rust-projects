// =============================================================================
// error.rs - Tipi di errore custom per Bullo CLI
// =============================================================================
//
// CONCETTI RUST IN QUESTO FILE:
//
// 1. `enum` con dati:
//    Ogni variante puo` contenere dati diversi. Esempio:
//    `Io(std::io::Error)` contiene un errore I/O originale
//    `NotFound(String)` contiene il path come stringa
//
// 2. `#[derive(Debug, thiserror::Error)]`:
//    - `Debug` = permette {:?} per stampa tecnica
//    - `thiserror::Error` = implementa automaticamente il trait `std::error::Error`
//
// 3. `#[error("...")]` = definisce il messaggio di errore per ogni variante.
//    `{0}` si riferisce al primo campo della variante.
//
// 4. `#[from]` = genera automaticamente la conversione da quel tipo di errore.
//    Quando usi `?` su un `std::io::Error`, Rust lo converte automaticamente
//    in `BulloError::Io(...)`.
//
// 5. `type Result<T> = std::result::Result<T, BulloError>;`
//    Type alias: crea un "soprannome" per un tipo complesso.
//    Cosi` invece di scrivere `Result<Vec<String>, BulloError>` ovunque,
//    scrivi semplicemente `Result<Vec<String>>`.
//
// =============================================================================

use std::path::PathBuf;

/// Errori possibili in Bullo CLI.
///
/// Ogni variante rappresenta una categoria diversa di errore.
/// `thiserror` genera automaticamente l'implementazione di `Display`
/// e `Error` trait basandosi sugli attributi `#[error(...)]`.
// Le varianti non usate verranno usate nelle fasi successive.
// Rimuoveremo questo allow quando saranno tutte implementate.
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum BulloError {
    /// Errore di I/O (lettura/scrittura file, accesso directory, etc.)
    #[error("Errore I/O: {0}")]
    Io(#[from] std::io::Error),
    //  ^^^^^^ `#[from]` genera: impl From<std::io::Error> for BulloError
    /// File o directory non trovato
    #[error("Non trovato: {0}")]
    NotFound(PathBuf),

    /// File o directory gia` esistente
    #[error("Gia` esistente: {0}")]
    AlreadyExists(PathBuf),

    /// Operazione non permessa (permessi insufficienti, etc.)
    #[error("Operazione non permessa: {0}")]
    PermissionDenied(String),

    /// Tipo di file non supportato per l'operazione richiesta
    #[error("Tipo non supportato: {0}")]
    UnsupportedType(String),
}

/// Type alias per Result con BulloError.
///
/// Permette di scrivere `Result<T>` invece di `Result<T, BulloError>`.
/// Ogni funzione che puo` fallire restituira` questo tipo.
pub type Result<T> = std::result::Result<T, BulloError>;
