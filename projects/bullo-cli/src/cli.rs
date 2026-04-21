// =============================================================================
// cli.rs - Definizione della struttura CLI con clap
// =============================================================================
//
// CONCETTI RUST IN QUESTO FILE:
//
// 1. `use` - importa tipi da altri moduli/crate (come import in Python/JS)
//
// 2. `#[derive(...)]` - chiede al compilatore di generare automaticamente
//    l'implementazione di certi "trait" (interfacce). Qui:
//    - `Parser`     -> genera il parsing degli argomenti CLI
//    - `Subcommand` -> genera il parsing dei sotto-comandi
//    - `Debug`      -> permette di stampare la struct con {:?}
//
// 3. `#[command(...)]` - attributi di clap che configurano il comportamento
//    del comando (nome, versione, descrizione, etc.)
//
// 4. `enum` - un tipo che puo` essere UNO tra diversi varianti.
//    In Rust gli enum sono molto piu` potenti che in altri linguaggi:
//    ogni variante puo` contenere dati diversi!
//
// 5. `String` vs `Option<String>`:
//    - `String` = campo obbligatorio
//    - `Option<String>` = campo opzionale (puo` essere Some("valore") o None)
//
// =============================================================================

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// Bullo CLI - Un file manager da terminale scritto in Rust
#[derive(Parser, Debug)]
#[command(name = "bullo")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Il sotto-comando da eseguire
    #[command(subcommand)]
    pub command: Commands,
}

/// Criterio di ordinamento per `ls`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SortBy {
    /// Ordina per nome
    Name,
    /// Ordina per dimensione
    Size,
    /// Ordina per data di modifica
    Date,
}

/// Tutti i comandi disponibili in Bullo CLI.
///
/// Ogni variante dell'enum corrisponde a un sotto-comando.
/// Per esempio: `bullo ls`, `bullo cp source dest`, etc.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Lista file e directory
    Ls {
        /// Path da listare (default: directory corrente)
        path: Option<PathBuf>,

        /// Output dettagliato con permessi, owner, dimensione
        #[arg(long)]
        long: bool,

        /// Criterio di ordinamento (default: name)
        #[arg(long, default_value = "name")]
        sort: SortBy,

        /// Ordine inverso
        #[arg(long)]
        reverse: bool,
    },

    /// Copia file o directory
    Cp {
        /// File/directory sorgente
        source: String,
        /// Destinazione
        dest: String,
    },

    /// Sposta o rinomina file/directory
    Mv {
        /// File/directory sorgente
        source: String,
        /// Destinazione
        dest: String,
    },

    /// Elimina file o directory
    Rm {
        /// File/directory da eliminare
        path: String,
    },

    /// Crea una nuova directory
    Mkdir {
        /// Nome della directory da creare
        path: String,
    },

    /// Mostra albero ricorsivo di directory
    Tree {
        /// Directory da visualizzare (default: directory corrente)
        path: Option<PathBuf>,

        /// Profondita` massima da visualizzare
        #[arg(long)]
        depth: Option<u32>,
    },

    /// Apri un file con il programma di default o specificato
    Open {
        /// File da aprire
        path: String,

        /// Programma specifico con cui aprire il file
        #[arg(long)]
        with: Option<String>,
    },
}
