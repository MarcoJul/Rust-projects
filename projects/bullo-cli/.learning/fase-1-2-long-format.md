# Fase 1.2 - Output Lungo (`--long`) e Path Opzionale

> In questa fase aggiungiamo il flag `--long` al comando `ls` per mostrare output dettagliato in stile Unix: permessi, owner, gruppo, dimensione e data. Imparerai conditional compilation, trait platform-specific e lookup di utenti/gruppi.

---

## 1. Flag Booleani in Clap

Aggiungere un flag booleano e` semplice: usa `#[arg(long)]`.

```rust
Ls {
    path: Option<PathBuf>,

    #[arg(long)]
    long: bool,
}
```

Questo crea il flag `--long`:
```bash
bullo ls          # long = false
bullo ls --long   # long = true
```

> **Perche` `#[arg(long)]` e non `#[arg(short)]`?**
> `--long` e` piu` leggibile di `-l` per un progetto didattico.
> In produzione, useresti entrambi: `#[arg(short = 'l', long)]`.

---

## 2. Conditional Compilation: `#[cfg(unix)]`

Alcune funzionalita` esistono solo su Unix/Linux/macOS. Rust ti permette di compilare codice diverso per ogni piattaforma:

```rust
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[cfg(unix)]
fn format_permissions(metadata: &fs::Metadata) -> String {
    // Codice Unix-specific
}

#[cfg(not(unix))]
fn format_permissions(metadata: &fs::Metadata) -> String {
    // Fallback per Windows
    "rw-rw-rw-".to_string()
}
```

### Come funziona `#[cfg(...)]`

Il compilatore **esclude completamente** il codice non pertinente alla piattaforma target. Non e` un `if` a runtime - e` una decisione a compile-time.

```rust
#[cfg(unix)]    // Solo su Linux/macOS/BSD
#[cfg(windows)] // Solo su Windows
#[cfg(target_os = "macos")] // Solo su macOS
#[cfg(not(unix))] // Tutto tranne Unix
```

> **Vantaggio:** zero overhead a runtime. Il codice non compilato non esiste nel binario.

---

## 3. `PermissionsExt` - Leggere i Permessi Unix

Su Unix, i permessi sono una **bitmask** octal (base 8):

```
0o755 = rwxr-xr-x
  ^^^
  | ||
  | |+-- Other: r-x (read + execute)
  +---- Owner: rwx (read + write + execute)
```

### I bit dei permessi

| Bit | Ottale | Significato |
|-----|--------|-------------|
| `0o400` | 400 | Owner read |
| `0o200` | 200 | Owner write |
| `0o100` | 100 | Owner execute |
| `0o040` | 040 | Group read |
| `0o020` | 020 | Group write |
| `0o010` | 010 | Group execute |
| `0o004` | 004 | Other read |
| `0o002` | 002 | Other write |
| `0o001` | 001 | Other execute |

### Leggere la bitmask

```rust
use std::os::unix::fs::PermissionsExt;

let mode = metadata.permissions().mode();

// Controlla se un bit e` attivo con AND (&)
if (mode & 0o400) != 0 {
    // Owner ha permesso di lettura
}
```

### Costruire la stringa `drwxr-xr-x`

```rust
fn format_permissions(metadata: &fs::Metadata) -> String {
    let mode = metadata.permissions().mode();

    // Primo carattere: tipo di file
    let type_char = if metadata.file_type().is_dir() {
        'd'
    } else if metadata.file_type().is_symlink() {
        'l'
    } else {
        '-'
    };

    // Controlla ogni bit e costruisci la stringa
    format!("{}{}{}{}{}{}{}{}{}{}",
        type_char,
        if (mode & 0o400) != 0 { 'r' } else { '-' },  // owner read
        if (mode & 0o200) != 0 { 'w' } else { '-' },  // owner write
        if (mode & 0o100) != 0 { 'x' } else { '-' },  // owner execute
        if (mode & 0o040) != 0 { 'r' } else { '-' },  // group read
        if (mode & 0o020) != 0 { 'w' } else { '-' },  // group write
        if (mode & 0o010) != 0 { 'x' } else { '-' },  // group execute
        if (mode & 0o004) != 0 { 'r' } else { '-' },  // other read
        if (mode & 0o002) != 0 { 'w' } else { '-' },  // other write
        if (mode & 0o001) != 0 { 'x' } else { '-' },  // other execute
    )
}
```

> **Perche` 0o e non 0x?**
> `0o` = ottale (base 8), usato per i permessi Unix
> `0x` = esadecimale (base 16), usato per indirizzi di memoria
> I permessi sono per tradizione in ottale perche` ogni cifra = 3 bit

---

## 4. `MetadataExt` - UID e GID

Oltre ai permessi, `MetadataExt` da` accesso a:

```rust
use std::os::unix::fs::MetadataExt;

let uid = metadata.uid();   // User ID del proprietario (u32)
let gid = metadata.gid();   // Group ID del gruppo (u32)
```

Ma UID e GID sono solo numeri. Per convertirli in nomi serve una crate esterna.

---

## 5. `uzers` - Lookup Utenti e Gruppi

La crate `uzers` converte UID/GID in nomi leggibili:

```rust
use uzers::{get_user_by_uid, get_group_by_gid};

// UID -> nome utente
let uid = metadata.uid();
match get_user_by_uid(uid) {
    Some(user) => println!("Owner: {}", user.name().to_string_lossy()),
    None => println!("Owner: {}", uid),  // fallback se utente non trovato
}

// GID -> nome gruppo
let gid = metadata.gid();
match get_group_by_gid(gid) {
    Some(group) => println!("Group: {}", group.name().to_string_lossy()),
    None => println!("Group: {}", gid),
}
```

### Perche` `to_string_lossy()`?

I nomi utente/gruppo sono `OsString` (come i nomi file). Potrebbero contenere caratteri non-UTF8, quindi usiamo `to_string_lossy()` per una conversione sicura.

### Perche` `uzers` e non `users`?

La crate originale `users` non e` piu` mantenuta. `uzers` e` il fork attivo con le stesse API.

---

## 6. Due Strutture per Due Modalita`

Abbiamo separato i dati in due struct diverse:

```rust
// Modalita` normale: solo info essenziali
struct EntryInfo {
    name: String,
    size: String,
    modified: String,
    file_type: FileType,
}

// Modalita` --long: info complete
struct LongEntryInfo {
    permissions: String,
    owner: String,
    group: String,
    size: String,
    modified: String,
    name: String,
    file_type: FileType,
}
```

> **Perche` non una struct con `Option`?**
> Due struct separate e` piu` pulito:
> - Niente `Option<String>` sparsi
> - Ogni struct ha esattamente i campi che servono
> - Il compilatore ti aiuta a non dimenticare campi

---

## 7. Dispatch nel `execute()`

La funzione `execute()` ora riceve il flag `long` e sceglie il percorso:

```rust
pub fn execute(path: Option<PathBuf>, long: bool) -> Result<()> {
    // ... validazione path ...

    if long {
        // Raccogli LongEntryInfo
        let mut infos: Vec<LongEntryInfo> = Vec::new();
        // ... popola infos ...
        print_long_listing(&infos);
    } else {
        // Raccogli EntryInfo
        let mut infos: Vec<EntryInfo> = Vec::new();
        // ... popola infos ...
        print_listing(&infos);
    }

    Ok(())
}
```

---

## 8. Formattare Output Lungo

Il formato lungo allinea colonne come `ls -l`:

```
Permessi   OWNER GROUP Dimensione  Modifica          Nome
-rw-r--r-- csdev csdev        154  2026-04-07 11:45  .gitignore
drwxr-xr-x csdev csdev         78  2026-04-17 12:31  .learning
```

### Larghezze dinamiche

```rust
let max_owner = infos.iter()
    .map(|e| e.owner.len())
    .max()
    .unwrap_or(0)
    .max(5);  // minimo 5 per header "OWNER"
```

`.max(5)` assicura che la colonna sia larga almeno quanto l'header, anche se tutti i nomi sono corti.

---

## 9. Fallback Cross-Platform

Per Windows, forniamo versioni semplificate:

```rust
#[cfg(not(unix))]
fn format_permissions(metadata: &fs::Metadata) -> String {
    if metadata.permissions().readonly() {
        "r--r--r--".to_string()
    } else {
        "rw-rw-rw-".to_string()
    }
}

#[cfg(not(unix))]
fn format_owner(_metadata: &fs::Metadata) -> String {
    "unknown".to_string()
}

#[cfg(not(unix))]
fn format_group(_metadata: &fs::Metadata) -> String {
    "unknown".to_string()
}
```

> **Nota:** Su Windows `Metadata` non ha `uid()` o `gid()`.
> Il fallback permette al codice di compilare su tutte le piattaforme,
> anche se l'output e` meno dettagliato.

---

## 10. Test con File Temporanei

Testare i permessi richiede file reali:

```rust
#[test]
#[cfg(unix)]
fn test_format_permissions() {
    // Crea file temporaneo
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("bullo_test_perms");
    std::fs::write(&test_file, "test").unwrap();

    // Leggi metadati e testa
    let metadata = std::fs::metadata(&test_file).unwrap();
    let perms = format_permissions(&metadata);

    assert_eq!(perms.len(), 10);  // sempre 10 caratteri
    assert_eq!(&perms[0..1], "-"); // file, non directory

    // Cleanup
    std::fs::remove_file(&test_file).unwrap();
}
```

### Pattern: setup → test → cleanup

```rust
// 1. Setup: crea risorse necessarie
let file = create_temp_file();

// 2. Test: verifica il comportamento
assert_eq!(function_under_test(&file), expected);

// 3. Cleanup: rimuovi risorse
std::fs::remove_file(&file).unwrap();
```

> **Attenzione:** se il test fallisce prima del cleanup, il file temporaneo rimane.
> Per test piu` robusti, usa `tempfile` crate che fa cleanup automatico.

---

## 11. Numeri Ottaletti in Rust

I permessi Unix usano la base 8 (ottale). Rust supporta letterali ottaletti:

```rust
0o755  // = 493 in decimale = rwxr-xr-x
0o644  // = 420 in decimale = rw-r--r--
0o000  // = 0   in decimale = ---------
```

### Confronto con AND bit-a-bit

```rust
let mode = 0o755;

(mode & 0o400) != 0  // true  -> owner ha read
(mode & 0o200) != 0  // true  -> owner ha write
(mode & 0o100) != 0  // true  -> owner ha execute
(mode & 0o040) != 0  // true  -> group ha read
(mode & 0o020) != 0  // false -> group NON ha write
(mode & 0o010) != 0  // true  -> group ha execute
```

> **Come funziona AND (`&`)?**
> `0o755 & 0o400 = 0o400` (il bit e` attivo)
> `0o755 & 0o020 = 0o000` (il bit NON e` attivo)
> Se il risultato e` diverso da 0, il bit e` impostato.

---

## Riepilogo: Cosa Abbiamo Aggiunto

```
bullo ls           -> output compatto (tipo, nome, dimensione, data)
bullo ls --long    -> output dettagliato (permessi, owner, group, size, data, nome)
bullo ls /tmp      -> path esplicito
bullo ls /tmp --long -> combinazione di entrambi
```

---

## Riepilogo Rapido

| Concetto | Keyword/Simbolo | In una riga |
|----------|----------------|-------------|
| Flag booleano | `#[arg(long)]` | Crea flag `--nome` in clap |
| Conditional compilation | `#[cfg(unix)]` | Compila solo su Unix |
| Fallback platform | `#[cfg(not(unix))]` | Compila su non-Unix |
| Permessi Unix | `PermissionsExt` | `.mode()` ritorna bitmask |
| Bitmask AND | `mode & 0o400` | Controlla se un bit e` attivo |
| Letterale ottale | `0o755` | Base 8 per permessi |
| UID/GID | `MetadataExt` | `.uid()`, `.gid()` |
| Lookup utente | `uzers` | UID → nome utente |
| Lookup gruppo | `uzers` | GID → nome gruppo |
| Test con file | `std::fs` | Crea/testa/cleanup file |
