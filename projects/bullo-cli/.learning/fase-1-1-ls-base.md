# Fase 1.1 - Il Comando `ls`: Liste File e Directory

> In questa fase implementiamo il comando `bullo ls` che lista il contenuto di una directory.
> Imparerai a lavorare con il filesystem, iterare su directory, formattare output e gestire errori pratici.

---

## 1. `std::fs::read_dir()` - Leggere una Directory

La funzione `read_dir()` e` il punto di ingresso per listare file. Ritorna un **iteratore** su `Result<DirEntry>`.

```rust
use std::fs;

let entries = fs::read_dir("/tmp")?;
// entries e` un ReadDir, che implementa Iterator<Item = Result<DirEntry>>
```

> **Perche` Result<DirEntry>` e non solo `DirEntry`?**
> Perche` ogni entry potrebbe avere problemi di accesso (permessi, symlink rotti, etc.).
> Rust ti forza a gestire questi casi uno per uno.

### Come si usa in pratica

```rust
for entry in fs::read_dir(path)? {
    let entry = entry?;  // Qui gestiamo l'errore per ogni singola entry
    // ora `entry` e` un DirEntry valido
}
```

Il doppio `?` confonde all'inizio:
- Il primo `?` su `read_dir()` gestisce errori di apertura directory
- Il secondo `?` su `entry` gestisce errori per ogni singolo file

---

## 2. `DirEntry` - Informazioni Base su un File

`DirEntry` ti da` accesso a informazioni **senza** caricare tutti i metadati (piu` efficiente).

```rust
let entry: DirEntry = ...;

// Nome del file (ritorna OsString, non String!)
let name = entry.file_name();

// Tipo (file, directory, symlink) - anche questo e` un Result!
let file_type = entry.file_type()?;

// Metadati completi (dimensione, date, permessi)
let metadata = entry.metadata()?;
```

### Convertire `OsString` in `String`

`file_name()` ritorna `OsString` perche` i nomi file possono contenere caratteri non-UTF8.
Per convertirlo in `String` usiamo `to_string_lossy()`:

```rust
let name: String = entry.file_name().to_string_lossy().to_string();
//                                   ^^^^^^^^^^^^^^^^
//                                   Se non e` UTF-8, sostituisce i caratteri invalidi con 
```

> **Perche` non `unwrap()`?** Perche` su Linux i nomi file possono essere bytes arbitrari.
> `to_string_lossy()` e` sicuro e non panica mai.

---

## 3. `FileType` - Sapere se e` File, Directory o Symlink

```rust
let file_type = entry.file_type()?;

if file_type.is_dir() {
    println!("E' una directory");
} else if file_type.is_file() {
    println!("E' un file");
} else if file_type.is_symlink() {
    println!("E' un link simbolico");
}
```

> **Attenzione:** `is_symlink()` e` separato da `is_file()` e `is_dir()`.
> Un symlink puo` puntare a un file O a una directory.
> Nel nostro codice controlliamo `is_symlink()` PRIMA degli altri per distinguerlo.

---

## 4. `Metadata` - Dimensione e Date

```rust
let metadata = entry.metadata()?;

// Dimensione in bytes (u64)
let size: u64 = metadata.len();

// Data di ultima modifica (SystemTime)
let modified: SystemTime = metadata.modified()?;

// Shortcut per tipo (legge i metadati internamente)
if metadata.is_dir() { ... }
```

### `SystemTime` non ha formattazione!

`SystemTime` e` un tipo "grezzo" - sa solo rappresentare un momento nel tempo,
ma non sa come stamparlo. Per formattarlo dobbiamo convertirlo in `DateTime`:

```rust
use chrono::{DateTime, Local};

let modified: SystemTime = metadata.modified()?;
let datetime: DateTime<Local> = DateTime::from(modified);
let formatted = datetime.format("%Y-%m-%d %H:%M").to_string();
// Risultato: "2024-01-15 14:30"
```

---

## 5. `PathBuf` vs `String` per i Path

Abbiamo cambiato `Option<String>` in `Option<PathBuf>` nel CLI. Perche`?

| Tipo | Uso |
|------|-----|
| `String` | Testo generico |
| `PathBuf` | Path del filesystem (gestisce `/` vs `\`, encoding nativo) |

```rust
use std::path::PathBuf;

// Da String a PathBuf
let path: PathBuf = PathBuf::from("/tmp/test");

// PathBuf ha metodi utili per il filesystem
path.exists()      // il file esiste?
path.is_dir()      // e` una directory?
path.is_file()     // e` un file?
path.display()     // stampa leggibile
```

> **Regola:** Usa `PathBuf` quando lavori con il filesystem.
> Usa `String` per testo generico (nomi, descrizioni, etc.).

---

## 6. Validazione del Path

Prima di listare, verifichiamo che il path sia valido:

```rust
if !target.exists() {
    return Err(BulloError::NotFound(target));
}

if !target.is_dir() {
    return Err(BulloError::UnsupportedType(format!(
        "'{}' non e` una directory",
        target.display()
    )));
}
```

> **Perche` due check separati?**
> - `exists()` cattura: file non trovato, path inesistente
> - `is_dir()` cattura: l'utente ha passato un file invece di una directory
>
> Errori diversi = messaggi diversi = utente piu` felice.

---

## 7. Raccogliere Dati in un `Vec`

Invece di stampare subito, raccogliamo tutte le entry in un `Vec` per:
1. Ordinarle prima di stampare
2. Calcolare le larghezze massime delle colonne

```rust
let mut infos: Vec<EntryInfo> = Vec::new();

for entry in entries {
    // ... estrai dati ...
    infos.push(EntryInfo { name, size, modified, file_type });
}

// Ora ordina per nome (case-insensitive)
infos.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
```

### `sort_by()` con closure

```rust
infos.sort_by(|a, b| a.name.cmp(&b.name));
//             ^^^^^^^^ closure che compara due elementi
```

La closure `|a, b|` prende due riferimenti e ritorna un `Ordering`:
- `Ordering::Less` -> `a` viene prima di `b`
- `Ordering::Equal` -> sono uguali
- `Ordering::Greater` -> `a` viene dopo di `b`

---

## 8. Formattare Dimensioni Umane

Convertire bytes in KB/MB/GB rende l'output leggibile:

```rust
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    match bytes {
        0 => "0 B".to_string(),
        b if b < KB => format!("{b} B"),
        b if b < MB => format!("{:.1} KB", b as f64 / KB as f64),
        b if b < GB => format!("{:.1} MB", b as f64 / MB as f64),
        _ => format!("{:.2} GB", b as f64 / GB as f64),
    }
}
```

### Match con guardie (`if`)

```rust
match bytes {
    b if b < KB => ...   // "se bytes < 1024, usa questo ramo"
    b if b < MB => ...   // "altrimenti se < 1MB, usa questo"
}
```

Le guardie (`if`) permettono condizioni complesse nel pattern matching.

### Formattazione numeri decimali

```rust
format!("{:.1} KB", 1.5)   // "1.5 KB"  (1 decimale)
format!("{:.2} GB", 1.5)   // "1.50 GB" (2 decimali)
format!("{}", 42)           // "42"     (default)
```

---

## 9. Output in Colonne Allineate

Per un output leggibile, calcoliamo la larghezza massima di ogni colonna:

```rust
let max_name = infos.iter().map(|e| e.name.len()).max().unwrap_or(0);
let max_size = infos.iter().map(|e| e.size.len()).max().unwrap_or(0);
```

### Spiegazione passo-passo

```rust
infos.iter()              // Iterator<&EntryInfo>
    .map(|e| e.name.len()) // Iterator<usize> - estrae lunghezze nomi
    .max()                 // Option<usize> - la lunghezza massima
    .unwrap_or(0)          // usize - 0 se la lista e` vuota
```

### Stampare con padding

```rust
println!(
    "{:<4} {:<width_name$} {:>width_size$}  {}",
    "Tipo",
    "Nome",
    "Dimensione",
    "Modifica",
    width_name = max_name,
    width_size = max_size,
);
```

- `{:<4}` = allinea a sinistra, larghezza 4
- `{:>width_size$}` = allinea a destra, larghezza dinamica
- `{:<width_name$}` = allinea a sinistra, larghezza dinamica

> **Trucco:** `{:>}` allinea a destra (numeri), `{:<}` allinea a sinistra (testo).

---

## 10. Test Unitari

Testare le funzioni pure e` facile e veloce:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
    }
}
```

### Perche` testare `format_size` e non `execute`?

- `format_size` e` **pura**: stesso input = stesso output, nessun effetto collaterale
- `execute` dipende dal filesystem: testarla richiede creare directory temporanee
- Testare le funzioni pure prima, poi aggiungeremo test di integrazione

---

## 11. `DateTime<Local>` - Timezone Locale

```rust
use chrono::{DateTime, Local};

let now: DateTime<Local> = Local::now();
let from_system: DateTime<Local> = DateTime::from(system_time);
```

`Local` usa il fuso orario del sistema operativo. Se sei in Italia, usi CET/CEST.

### Pattern di formattazione

| Pattern | Risultato | Esempio |
|---------|-----------|---------|
| `%Y-%m-%d` | Anno-Mese-Giorno | `2024-01-15` |
| `%H:%M` | Ora:Minuti | `14:30` |
| `%Y-%m-%d %H:%M:%S` | Completo | `2024-01-15 14:30:45` |
| `%d/%m/%Y` | Italiano | `15/01/2024` |

---

## Riepilogo: Il Flusso di `ls`

```
1. Determina target (path fornito o directory corrente)
2. Valida: esiste? e` una directory?
3. Leggi entry con fs::read_dir()
4. Per ogni entry:
   - Estrai nome, tipo, dimensione, data
   - Crea EntryInfo
5. Ordina per nome
6. Calcola larghezze colonne
7. Stampa header + entries allineate
```

---

## Riepilogo Rapido

| Concetto | Keyword/Simbolo | In una riga |
|----------|----------------|-------------|
| Leggere directory | `fs::read_dir()` | Iteratore su `Result<DirEntry>` |
| Entry singola | `DirEntry` | Nome, tipo, metadati |
| Tipo file | `FileType` | File, Dir, Symlink |
| Metadati | `Metadata` | Dimensione, date, permessi |
| Path nativo | `PathBuf` | Gestisce `/` e `\` automaticamente |
| Ordinare | `.sort_by()` | Closure di comparazione |
| Formattare numeri | `{:.1}`, `{:.2}` | Decimali controllati |
| Padding | `{:<10}`, `{:>10}` | Allinea sinistra/destra |
| Timezone locale | `DateTime<Local>` | Usa fuso orario del sistema |
| Formattare date | `.format("%Y-%m-%d")` | Pattern strftime-like |
