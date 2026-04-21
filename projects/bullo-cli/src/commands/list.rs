// =============================================================================
// commands/list.rs - Implementazione del comando `ls`
// =============================================================================
//
// CONCETTI RUST IN QUESTO FILE:
//
// 1. `std::fs::read_dir()` - legge il contenuto di una directory.
//    Ritorna un `ReadDir` che e` un iteratore su `Result<DirEntry>`.
//    Ogni `DirEntry` rappresenta un file o directory nella cartella.
//
// 2. `DirEntry` - contiene info base senza leggere i metadati completi.
//    Metodi principali:
//    - `.file_name()` -> OsString (nome del file)
//    - `.file_type()` -> Result<FileType> (file, dir, o symlink)
//    - `.metadata()` -> Result<Metadata> (dimensione, permessi, date)
//
// 3. `FileType` - enum che indica il tipo di entry:
//    - `.is_file()` -> true se e` un file regolare
//    - `.is_dir()` -> true se e` una directory
//    - `.is_symlink()` -> true se e` un link simbolico
//
// 4. `Metadata` - informazioni dettagliate sul file:
//    - `.len()` -> dimensione in bytes (u64)
//    - `.modified()` -> data ultima modifica (SystemTime)
//    - `.is_dir()` / `.is_file()` -> shortcut per il tipo
//
// 5. `SystemTime` -> `chrono::DateTime` conversione:
//    `SystemTime` non ha metodi di formattazione. Lo convertiamo in
//    `DateTime<Local>` per usare `format()` con pattern tipo strftime.
//
// 6. `for entry in read_dir(path)?` - iterazione con propagazione errore.
//    Il `?` qui gestisce errori di lettura della directory.
//    Ogni `entry` e` un `Result<DirEntry>`, quindi serve un altro `?` dentro.
//
// 7. `format_size()` - funzione helper per formattare bytes in unita` umane.
//    Usa match su range per scegliere l'unita` giusta (B, KB, MB, GB, TB).
//
// 8. `std::os::unix::fs::PermissionsExt` - trait Unix-specific per leggere
//    la bitmask dei permessi (es: 0o755 = rwxr-xr-x).
//    Disponibile solo su Unix/Linux/macOS, non su Windows.
//
// 9. `uzers` crate - lookup uid/gid -> nome utente/gruppo.
//    Cross-platform alternativa a chiamate di sistema specifiche.
//
// =============================================================================

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use owo_colors::OwoColorize;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use uzers::{get_group_by_gid, get_user_by_uid};

use crate::cli::SortBy;
use crate::error::{BulloError, Result};

/// Tipo di file riconosciuto da `ls`.
#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Directory,
    File,
    Symlink,
}

/// Informazioni base su un entry (modalita` normale).
struct EntryInfo {
    name: String,
    size: String,
    modified: String,
    file_type: FileType,
    // Raw values for sorting
    size_bytes: u64,
    modified_time: SystemTime,
}

/// Informazioni dettagliate su un entry (modalita` --long).
#[allow(dead_code)]
struct LongEntryInfo {
    permissions: String,
    owner: String,
    group: String,
    size: String,
    modified: String,
    name: String,
    file_type: FileType,
    // Raw values for sorting
    size_bytes: u64,
    modified_time: SystemTime,
}

/// Esegue il comando `ls`: lista il contenuto di una directory.
///
/// # Argomenti
/// * `path` - Directory da listare. Se `None`, usa la directory corrente.
/// * `long` - Se true, mostra output dettagliato (permessi, owner, etc.).
/// * `sort_by` - Criterio di ordinamento (name, size, date).
/// * `reverse` - Se true, inverte l'ordinamento.
///
/// # Errors
/// - `BulloError::NotFound` se il path non esiste
/// - `BulloError::UnsupportedType` se il path non e` una directory
/// - `BulloError::Io` per altri errori di I/O
pub fn execute(path: Option<PathBuf>, long: bool, sort_by: SortBy, reverse: bool) -> Result<()> {
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

    let entries = fs::read_dir(&target)?;

    if long {
        let mut infos: Vec<LongEntryInfo> = Vec::new();

        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let file_type = entry.file_type()?;

            let entry_type = if file_type.is_symlink() {
                FileType::Symlink
            } else if file_type.is_dir() {
                FileType::Directory
            } else {
                FileType::File
            };

            let permissions = format_permissions(&metadata);
            let owner = format_owner(&metadata);
            let group = format_group(&metadata);
            let size = format!("{:>8}", metadata.len());
            let modified = format_date(metadata.modified()?);
            let name = entry.file_name().to_string_lossy().to_string();
            let size_bytes = metadata.len();
            let modified_time = metadata.modified()?;

            infos.push(LongEntryInfo {
                permissions,
                owner,
                group,
                size,
                modified,
                name,
                file_type: entry_type,
                size_bytes,
                modified_time,
            });
        }

        sort_entries_long(&mut infos, &sort_by, reverse);
        print_long_listing(&infos);
    } else {
        let mut infos: Vec<EntryInfo> = Vec::new();

        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let metadata = entry.metadata()?;
            let file_type = entry.file_type()?;

            let entry_type = if file_type.is_symlink() {
                FileType::Symlink
            } else if file_type.is_dir() {
                FileType::Directory
            } else {
                FileType::File
            };

            let size = format_size(metadata.len());
            let modified = format_date(metadata.modified()?);
            let size_bytes = metadata.len();
            let modified_time = metadata.modified()?;

            infos.push(EntryInfo {
                name: file_name,
                size,
                modified,
                file_type: entry_type,
                size_bytes,
                modified_time,
            });
        }

        sort_entries(&mut infos, &sort_by, reverse);
        print_listing(&infos);
    }

    Ok(())
}

/// Formatta una dimensione in bytes in unita` umane.
///
/// # Esempi
/// ```
/// format_size(0)          -> "0 B"
/// format_size(512)        -> "512 B"
/// format_size(1024)       -> "1.0 KB"
/// format_size(1_500_000)  -> "1.4 MB"
/// format_size(1_000_000_000) -> "953.7 MB"
/// ```
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    match bytes {
        0 => "0 B".to_string(),
        b if b < KB => format!("{b} B"),
        b if b < MB => format!("{:.1} KB", b as f64 / KB as f64),
        b if b < GB => format!("{:.1} MB", b as f64 / MB as f64),
        b if b < TB => format!("{:.2} GB", b as f64 / GB as f64),
        _ => format!("{:.2} TB", bytes as f64 / TB as f64),
    }
}

/// Formatta un SystemTime in data/ora ISO locale.
///
/// Converte `SystemTime` in `DateTime<Local>` e usa il pattern:
/// `%Y-%m-%d %H:%M` -> "2024-01-15 14:30"
fn format_date(time: std::time::SystemTime) -> String {
    let datetime: DateTime<Local> = DateTime::from(time);
    datetime.format("%Y-%m-%d %H:%M").to_string()
}

/// Ordina le entry normali per criterio specificato.
fn sort_entries(infos: &mut [EntryInfo], sort_by: &SortBy, reverse: bool) {
    infos.sort_by(|a, b| {
        let ordering = match sort_by {
            SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortBy::Size => a.size_bytes.cmp(&b.size_bytes),
            SortBy::Date => a.modified_time.cmp(&b.modified_time),
        };
        if reverse {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

/// Ordina le entry lunghe per criterio specificato.
fn sort_entries_long(infos: &mut [LongEntryInfo], sort_by: &SortBy, reverse: bool) {
    infos.sort_by(|a, b| {
        let ordering = match sort_by {
            SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortBy::Size => a.size_bytes.cmp(&b.size_bytes),
            SortBy::Date => a.modified_time.cmp(&b.modified_time),
        };
        if reverse {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

/// Formatta i permessi in stile Unix (es: `drwxr-xr-x`).
///
/// Usa `PermissionsExt::mode()` per ottenere la bitmask octal.
/// Il primo carattere indica il tipo: `d` = directory, `l` = symlink, `-` = file.
#[cfg(unix)]
fn format_permissions(metadata: &fs::Metadata) -> String {
    let mode = metadata.permissions().mode();
    let file_type = metadata.file_type();

    let type_char = if file_type.is_dir() {
        'd'
    } else if file_type.is_symlink() {
        'l'
    } else {
        '-'
    };

    let bits = [
        (mode & 0o400) != 0,
        (mode & 0o200) != 0,
        (mode & 0o100) != 0,
        (mode & 0o040) != 0,
        (mode & 0o020) != 0,
        (mode & 0o010) != 0,
        (mode & 0o004) != 0,
        (mode & 0o002) != 0,
        (mode & 0o001) != 0,
    ];

    let mut result = String::with_capacity(10);
    result.push(type_char);
    for &bit in &bits {
        result.push(if bit { 'r' } else { '-' });
    }

    // Fix: i bit rwx devono essere nelle posizioni giuste
    // Owner: rwx = 4,2,1 -> posizioni 1,2,3
    // Group: rwx = 4,2,1 -> posizioni 4,5,6
    // Other: rwx = 4,2,1 -> posizioni 7,8,9
    let mut chars: Vec<char> = result.chars().collect();
    // Owner read (0o400)
    chars[1] = if (mode & 0o400) != 0 { 'r' } else { '-' };
    // Owner write (0o200)
    chars[2] = if (mode & 0o200) != 0 { 'w' } else { '-' };
    // Owner execute (0o100)
    chars[3] = if (mode & 0o100) != 0 { 'x' } else { '-' };
    // Group read (0o040)
    chars[4] = if (mode & 0o040) != 0 { 'r' } else { '-' };
    // Group write (0o020)
    chars[5] = if (mode & 0o020) != 0 { 'w' } else { '-' };
    // Group execute (0o010)
    chars[6] = if (mode & 0o010) != 0 { 'x' } else { '-' };
    // Other read (0o004)
    chars[7] = if (mode & 0o004) != 0 { 'r' } else { '-' };
    // Other write (0o002)
    chars[8] = if (mode & 0o002) != 0 { 'w' } else { '-' };
    // Other execute (0o001)
    chars[9] = if (mode & 0o001) != 0 { 'x' } else { '-' };

    chars.into_iter().collect()
}

/// Fallback per Windows (non usa PermissionsExt).
#[cfg(not(unix))]
fn format_permissions(metadata: &fs::Metadata) -> String {
    if metadata.permissions().readonly() {
        "r--r--r--".to_string()
    } else {
        "rw-rw-rw-".to_string()
    }
}

/// Formatta il nome dell'owner usando `uzers`.
#[cfg(unix)]
fn format_owner(metadata: &fs::Metadata) -> String {
    let uid = metadata.uid();
    match get_user_by_uid(uid) {
        Some(user) => user.name().to_string_lossy().to_string(),
        None => uid.to_string(),
    }
}

/// Fallback per Windows.
#[cfg(not(unix))]
fn format_owner(_metadata: &fs::Metadata) -> String {
    "unknown".to_string()
}

/// Formatta il nome del gruppo usando `uzers`.
#[cfg(unix)]
fn format_group(metadata: &fs::Metadata) -> String {
    let gid = metadata.gid();
    match get_group_by_gid(gid) {
        Some(group) => group.name().to_string_lossy().to_string(),
        None => gid.to_string(),
    }
}

/// Fallback per Windows.
#[cfg(not(unix))]
fn format_group(_metadata: &fs::Metadata) -> String {
    "unknown".to_string()
}

/// Applica colori al nome del file in base al tipo.
///
/// - Directory: blu
/// - File eseguibili: verde
/// - Symlink: cyan
/// - File normale: nessun colore
fn colorize_name(name: &str, file_type: FileType, is_executable: bool) -> String {
    match file_type {
        FileType::Directory => name.blue().to_string(),
        FileType::Symlink => name.cyan().to_string(),
        FileType::File => {
            if is_executable {
                name.green().to_string()
            } else {
                name.to_string()
            }
        }
    }
}

/// Controlla se un file e` eseguibile (Unix: bit execute impostato).
#[cfg(unix)]
#[allow(dead_code)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    metadata.permissions().mode() & 0o111 != 0
}

/// Fallback per Windows (nessun concetto di execute bit).
#[cfg(not(unix))]
#[allow(dead_code)]
fn is_executable(_metadata: &fs::Metadata) -> bool {
    false
}

/// Stampa il listing normale (colonne allineate).
///
/// Formato: [TIPO]  NOME                    DIMENSIONE    DATA MODIFICA
/// I nomi sono colorati: directory=blu, eseguibili=verde, symlink=cyan
fn print_listing(infos: &[EntryInfo]) {
    if infos.is_empty() {
        println!("(directory vuota)");
        return;
    }

    // Calcola larghezze massime (senza codici ANSI)
    let max_name = infos.iter().map(|e| e.name.len()).max().unwrap_or(0).max(4); // "Nome"
    let max_size = infos
        .iter()
        .map(|e| e.size.len())
        .max()
        .unwrap_or(0)
        .max(10); // "Dimensione"
    let max_date = infos
        .iter()
        .map(|e| e.modified.len())
        .max()
        .unwrap_or(0)
        .max(8); // "Modifica"

    // Header
    println!(
        "{:<4} {:<width_name$} {:>width_size$}  {:<width_date$}",
        "Tipo",
        "Nome",
        "Dimensione",
        "Modifica",
        width_name = max_name,
        width_size = max_size,
        width_date = max_date,
    );
    println!("{}", "-".repeat(max_name + max_size + max_date + 16));

    // Entries
    for info in infos {
        let type_label = match info.file_type {
            FileType::Directory => "[D] ",
            FileType::File => "[F] ",
            FileType::Symlink => "[L] ",
        };

        let colored_name = colorize_name(&info.name, info.file_type, false);

        println!(
            "{:<4} {:<width_name$} {:>width_size$}  {:<width_date$}",
            type_label,
            colored_name,
            info.size,
            info.modified,
            width_name = max_name,
            width_size = max_size,
            width_date = max_date,
        );
    }

    println!("\n{} entry", infos.len());
}

/// Stampa il listing lungo (stile `ls -l`).
///
/// Formato: PERMESSI  OWNER  GROUP  DIMENSIONE  DATA MODIFICA  NOME
/// I nomi sono colorati: directory=blu, eseguibili=verde, symlink=cyan
fn print_long_listing(infos: &[LongEntryInfo]) {
    if infos.is_empty() {
        println!("(directory vuota)");
        return;
    }

    // Calcola larghezze massime (senza codici ANSI)
    let max_name = infos.iter().map(|e| e.name.len()).max().unwrap_or(0);
    let max_owner = infos
        .iter()
        .map(|e| e.owner.len())
        .max()
        .unwrap_or(0)
        .max(5); // "OWNER"
    let max_group = infos
        .iter()
        .map(|e| e.group.len())
        .max()
        .unwrap_or(0)
        .max(5); // "GROUP"

    // Header
    println!(
        "{:<10} {:<width_owner$} {:<width_group$} {:>10}  {:<16}  Nome",
        "Permessi",
        "OWNER",
        "GROUP",
        "Dimensione",
        "Modifica",
        width_owner = max_owner,
        width_group = max_group,
    );
    println!("{}", "-".repeat(max_name + max_owner + max_group + 52));

    // Entries
    for info in infos {
        let is_exec = info.permissions.contains('x');
        let colored_name = colorize_name(&info.name, info.file_type, is_exec);

        println!(
            "{:<10} {:<width_owner$} {:<width_group$} {:>10}  {:<16}  {}",
            info.permissions,
            info.owner,
            info.group,
            info.size,
            info.modified,
            colored_name,
            width_owner = max_owner,
            width_group = max_group,
        );
    }

    println!("\n{} entry", infos.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(10240), "10.0 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1_048_576), "1.0 MB");
        assert_eq!(format_size(1_572_864), "1.5 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_format_size_terabytes() {
        assert_eq!(format_size(1_099_511_627_776), "1.00 TB");
    }

    #[test]
    fn test_format_date() {
        let now = std::time::SystemTime::now();
        let formatted = format_date(now);
        // Deve contenere anno-mese-giorno ora:minuti
        assert!(formatted.contains('-'));
        assert!(formatted.contains(':'));
    }

    #[test]
    #[cfg(unix)]
    fn test_format_permissions() {
        // Test con un file temporaneo
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("bullo_test_perms");
        std::fs::write(&test_file, "test").unwrap();

        let metadata = std::fs::metadata(&test_file).unwrap();
        let perms = format_permissions(&metadata);

        // Deve essere lungo 10 caratteri
        assert_eq!(perms.len(), 10);
        // Primo char deve essere '-' (file)
        assert_eq!(&perms[0..1], "-");

        // Cleanup
        std::fs::remove_file(&test_file).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn test_format_permissions_directory() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("bullo_test_perms_dir");
        std::fs::create_dir(&test_dir).unwrap();

        let metadata = std::fs::metadata(&test_dir).unwrap();
        let perms = format_permissions(&metadata);

        assert_eq!(perms.len(), 10);
        assert_eq!(&perms[0..1], "d");

        // Cleanup
        std::fs::remove_dir(&test_dir).unwrap();
    }
}
