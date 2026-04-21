// =============================================================================
// commands/tree.rs - Implementazione del comando `tree`
// =============================================================================
//
// CONCETTI RUST IN QUESTO FILE:
//
// 1. Ricorsione - una funzione che chiama se stessa per attraversare
//    strutture ad albero (directory e sotto-directory).
//    Ogni chiamata ricorsiva deve avere un caso base per terminare.
//
// 2. `Vec<bool>` come stack di contesto - tiene traccia di quali livelli
//    dell'albero hanno ancora "fratelli" da visualizzare, per disegnare
//    correttamente le linee verticali (`│`) e i connettori (`├──`, `└──`).
//
// 3. `std::fs::read_dir()` + ordinamento - leggiamo ogni directory e
//    ordiniamo le entry: directory prima, file dopo, poi alfabetico.
//
// 4. `depth` parameter - limite alla ricorsione. Quando `current_depth >= max_depth`,
//    smettiamo di scendere. Previene recursione infinita e output enormi.
//
// 5. Pattern matching su `Result` - gestiamo entry non leggibili (permessi)
//    stampando un warning invece di crashare.
//
// =============================================================================

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{BulloError, Result};

/// Esegue il comando `tree`: visualizzazione ad albero ricorsiva.
///
/// # Argomenti
/// * `path` - Directory da visualizzare. Se `None`, usa la directory corrente.
/// * `max_depth` - Profondita` massima. Se `None`, nessuna limitazione.
///
/// # Errors
/// - `BulloError::NotFound` se il path non esiste
/// - `BulloError::UnsupportedType` se il path non e` una directory
/// - `BulloError::Io` per altri errori di I/O
pub fn execute(path: Option<PathBuf>, max_depth: Option<u32>) -> Result<()> {
    let target = match path {
        Some(p) => p,
        None => std::env::current_dir()?,
    };

    if !target.exists() {
        return Err(BulloError::NotFound(target));
    }

    if !target.is_dir() {
        return Err(BulloError::UnsupportedType(format!(
            "'{}' non e` una directory",
            target.display()
        )));
    }

    let depth_limit = max_depth.unwrap_or(u32::MAX);

    println!("{}", target.display());

    let mut dirs_count = 0u64;
    let mut files_count = 0u64;

    // Stack di contesto: true = ci sono ancora fratelli dopo questo
    let mut parent_prefixes: Vec<bool> = Vec::new();

    walk_dir(
        &target,
        &mut parent_prefixes,
        depth_limit,
        0,
        &mut dirs_count,
        &mut files_count,
    )?;

    println!("\n{} directories, {} files", dirs_count, files_count);

    Ok(())
}

/// Attraversa ricorsivamente una directory e stampa l'albero.
///
/// # Argomenti
/// * `dir` - Directory corrente da attraversare
/// * `parent_prefixes` - Stack di contesto per disegnare le linee
/// * `depth_limit` - Profondita` massima consentita
/// * `current_depth` - Profondita` attuale (0 = root)
/// * `dirs_count` - Contatore directory (mutabile)
/// * `files_count` - Contatore file (mutabile)
fn walk_dir(
    dir: &Path,
    parent_prefixes: &mut Vec<bool>,
    depth_limit: u32,
    current_depth: u32,
    dirs_count: &mut u64,
    files_count: &mut u64,
) -> Result<()> {
    if current_depth >= depth_limit {
        return Ok(());
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Errore leggendo '{}': {}", dir.display(), e);
            return Ok(());
        }
    };

    // Raccogli e ordina: directory prima, poi file, alfabetico
    let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    let total = sorted.len();

    for (i, entry) in sorted.iter().enumerate() {
        let is_last = i == total - 1;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

        // Costruisci il prefisso per questa riga
        let prefix = build_prefix(parent_prefixes);

        // Connettore: `└──` per l'ultimo, `├──` per gli altri
        let connector = if is_last { "└── " } else { "├── " };

        println!("{}{}{}", prefix, connector, name);

        if is_dir {
            *dirs_count += 1;

            // Aggiungi contesto per il prossimo livello
            parent_prefixes.push(!is_last);
            walk_dir(
                &entry.path(),
                parent_prefixes,
                depth_limit,
                current_depth + 1,
                dirs_count,
                files_count,
            )?;
            parent_prefixes.pop();
        } else {
            *files_count += 1;
        }
    }

    Ok(())
}

/// Costruisce la stringa di prefisso per una riga dell'albero.
///
/// Usa `│   ` per livelli che hanno ancora fratelli dopo,
/// e `    ` (spazi) per livelli che sono l'ultimo figlio.
fn build_prefix(parent_prefixes: &[bool]) -> String {
    let mut prefix = String::new();

    for &has_more in parent_prefixes {
        if has_more {
            prefix.push_str("│   ");
        } else {
            prefix.push_str("    ");
        }
    }

    prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prefix_empty() {
        let result = build_prefix(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_prefix_single_level() {
        let result = build_prefix(&[true]);
        assert_eq!(result, "│   ");
    }

    #[test]
    fn test_build_prefix_last_sibling() {
        let result = build_prefix(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_prefix_nested() {
        let result = build_prefix(&[true, false]);
        assert_eq!(result, "│       ");
    }

    #[test]
    fn test_build_prefix_deep() {
        let result = build_prefix(&[true, true, true]);
        assert_eq!(result, "│   │   │   ");
    }

    #[test]
    fn test_tree_with_depth_limit() {
        // Crea struttura temporanea
        let temp_dir = std::env::temp_dir().join("bullo_tree_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("a/b/c")).unwrap();
        std::fs::write(temp_dir.join("a/file.txt"), "test").unwrap();
        std::fs::write(temp_dir.join("a/b/file2.txt"), "test").unwrap();

        // Esegui con depth=1 (solo primo livello)
        execute(Some(temp_dir.clone()), Some(1)).unwrap();

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
