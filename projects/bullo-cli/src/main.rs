// =============================================================================
// main.rs - Punto di ingresso di Bullo CLI
// =============================================================================
//
// CONCETTI RUST IN QUESTO FILE:
//
// 1. `mod cli;` - dichiara che esiste un modulo chiamato "cli".
//    Rust cerchera` il file `src/cli.rs` (oppure `src/cli/mod.rs`).
//    Senza questa riga, il compilatore non sa che cli.rs esiste!
//
// 2. `use` - importa nomi specifici dal modulo per usarli senza prefisso.
//    `use cli::{Cli, Commands}` ci permette di scrivere `Cli` invece di `cli::Cli`
//
// 3. `Cli::parse()` - metodo generato automaticamente da `#[derive(Parser)]`.
//    Legge gli argomenti dalla command line e li converte nella struct Cli.
//    Se l'utente sbaglia, clap mostra automaticamente l'errore e l'help.
//
// 4. `match` - pattern matching esaustivo. Rust ti OBBLIGA a gestire
//    TUTTE le varianti dell'enum. Se ne aggiungi una nuova e dimentichi
//    di gestirla qui, il compilatore ti da` errore. Questa e` una delle
//    feature piu` potenti di Rust per prevenire bug!
//
// 5. `{}` nel match - destructuring. Quando una variante dell'enum contiene
//    dati (come `Ls { path }`), puoi estrarre i campi direttamente.
//
// =============================================================================

mod cli;
mod commands;
mod error;

use std::process;

use clap::Parser;

use cli::{Cli, Commands};

// =============================================================================
// main() ora gestisce gli errori in modo centralizzato.
//
// NUOVO CONCETTO: `run()` ritorna Result<()>. Se qualcosa va storto,
// l'errore "risale" fino a main(), che lo stampa e esce con codice 1.
//
// Perche` non facciamo `fn main() -> Result<()>` direttamente?
// Si potrebbe, ma il messaggio di errore di default e` brutto (usa Debug {:?}).
// Con questo pattern controlliamo noi come mostrare l'errore all'utente.
// =============================================================================

fn main() {
    if let Err(e) = run() {
        // `if let` e` come match ma per un solo caso.
        // Equivale a: match run() { Err(e) => {...}, Ok(()) => {} }
        eprintln!("Errore: {e}");
        //  ^^^^^^ eprintln! stampa su stderr (non stdout)
        process::exit(1);
    }
}

/// Logica principale dell'applicazione.
///
/// Separiamo la logica da main() per poter usare `?` ovunque.
/// main() fa solo la gestione dell'errore finale.
fn run() -> error::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ls {
            path,
            long,
            sort,
            reverse,
        } => {
            commands::list::execute(path, long, sort, reverse)?;
        }
        Commands::Cp {
            source: _source,
            dest: _dest,
        } => {
            println!("TODO: implementare cp");
        }
        Commands::Mv {
            source: _source,
            dest: _dest,
        } => {
            println!("TODO: implementare mv");
        }
        Commands::Rm { path: _path } => {
            println!("TODO: implementare rm");
        }
        Commands::Mkdir { path: _path } => {
            println!("TODO: implementare mkdir");
        }
        Commands::Tree { path, depth } => {
            commands::tree::execute(path, depth)?;
        }
        Commands::Open {
            path: _path,
            with: _with,
        } => {
            println!("TODO: implementare open");
        }
    }

    Ok(())
}
