// =============================================================================
// commands/list.rs - Implementazione del comando `ls`
// =============================================================================
//
// CONCETTI RUST IN QUESTO FILE:
//
// 1. `crate::error` - `crate` si riferisce alla "radice" del progetto.
//    E` come scrivere un path assoluto: `crate::error::Result` significa
//    "il tipo Result definito in src/error.rs".
//
// 2. Funzione che ritorna `Result<()>`:
//    - `()` e` il tipo "unit" (come void in C). Significa "nessun valore".
//    - `Result<()>` = "puo` avere successo (senza valore) o fallire con errore"
//
// 3. L'operatore `?`:
//    `env::current_dir()?` - se current_dir() ritorna Err, la funzione
//    ritorna immediatamente quell'errore. Se ritorna Ok, estrae il valore.
//    Funziona perche` `std::io::Error` ha `#[from]` nel nostro BulloError.
//
// =============================================================================

use std::env;
use std::path::PathBuf;

use crate::error::Result;

/// Esegue il comando `ls`: lista il contenuto di una directory.
///
/// # Argomenti
/// * `path` - Directory da listare. Se `None`, usa la directory corrente.
///
/// # Errors
/// Ritorna errore se la directory corrente non e` accessibile.
pub fn execute(path: Option<String>) -> Result<()> {
    // `?` qui converte automaticamente std::io::Error -> BulloError::Io
    // grazie al `#[from]` che abbiamo definito in error.rs
    let target: PathBuf = match path {
        Some(p) => PathBuf::from(p),
        None => env::current_dir()?,
        //                       ^ prima usavamo .expect() (crash!)
        //                         ora usiamo ? (propaga l'errore al chiamante)
    };

    println!("Contenuto di: {}", target.display());
    println!("{}", "-".repeat(40));
    println!("(listing non ancora implementato)");
    println!("Path target: {:?}", target);

    // Ok(()) = "tutto ok, nessun valore da ritornare"
    Ok(())
}
