# Fase 1.5 - Ordinamento (`--sort` e `--reverse`)

> In questa fase aggiungiamo il sorting al comando `ls` con `--sort name|size|date` e `--reverse`. Imparerai `ValueEnum`, closures, `Ordering`, e come separare dati grezzi da formattazione.

---

## 1. `ValueEnum` - Enum come Valori CLI

Clap puo` convertire automaticamente una stringa CLI in un enum Rust:

```rust
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SortBy {
    Name,
    Size,
    Date,
}
```

```bash
bullo ls --sort name    # SortBy::Name
bullo ls --sort size    # SortBy::Size
bullo ls --sort date    # SortBy::Date
bullo ls --sort foo     # Errore automatico: "invalid value 'foo'"
```

### Come Funziona

`ValueEnum` genera automaticamente:
- Parsing da stringa a enum
- Lista valori validi per l'help
- Messaggi di errore per valori invalidi

```
  --sort <SORT>  Criterio di ordinamento (default: name)
                 [default: name]
                 [possible values: name, size, date]
```

> **Perche` Copy + PartialEq + Eq?**
> `Copy` = passa per valore senza `.clone()`
> `PartialEq + Eq` = confronta varianti con `==` e in `match`

---

## 2. Separare Dati Grezzi da Formattazione

Per ordinare per dimensione, serve il valore in bytes (u64), non la stringa formattata ("1.5 KB"):

```rust
struct EntryInfo {
    name: String,            // per stampa
    size: String,            // per stampa: "1.5 KB"
    size_bytes: u64,         // per sorting: 1536
    modified: String,        // per stampa: "2024-01-15 14:30"
    modified_time: SystemTime, // per sorting
    file_type: FileType,
}
```

> **Regola:** conserva sempre i dati grezzi per operazioni come sorting/filtering.
> La formattazione e` solo per la presentazione.

---

## 3. `sort_by()` con Closures

```rust
infos.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//             closure che ritorna Ordering
```

### Come Funziona `sort_by`

```rust
fn sort_by<F>(&mut self, compare: F)
where
    F: FnMut(&T, &T) -> Ordering,
```

La closure riceve due riferimenti e deve ritornare:
- `Ordering::Less` → `a` viene prima
- `Ordering::Greater` → `b` viene prima
- `Ordering::Equal` → ordine indifferente

### Tre Modi di Confrontare

```rust
// Per nome: confronto stringhe case-insensitive
SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),

// Per dimensione: confronto numerico u64
SortBy::Size => a.size_bytes.cmp(&b.size_bytes),

// Per data: confronto SystemTime
SortBy::Date => a.modified_time.cmp(&b.modified_time),
```

---

## 4. `Ordering::reverse()`

Per invertire l'ordinamento:

```rust
let ordering = a.size_bytes.cmp(&b.size_bytes);
// Ordering::Less (piccolo prima)

let reversed = ordering.reverse();
// Ordering::Greater (grande prima)
```

### Implementazione

```rust
fn sort_entries(infos: &mut [EntryInfo], sort_by: &SortBy, reverse: bool) {
    infos.sort_by(|a, b| {
        let ordering = match sort_by {
            SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortBy::Size => a.size_bytes.cmp(&b.size_bytes),
            SortBy::Date => a.modified_time.cmp(&b.modified_time),
        };
        if reverse { ordering.reverse() } else { ordering }
    });
}
```

> **Perche` non due funzioni separate?**
> `Ordering::reverse()` e` piu` elegante e DRY.
> Una sola funzione gestisce entrambi i casi.

---

## 5. `&mut [T]` - Slice Mutabile

```rust
fn sort_entries(infos: &mut [EntryInfo], ...) { }
//                     ^^^^^^^^^^^^^^^^^^
//                     slice mutabile, non &mut Vec
```

### Slice vs Vec

| Tipo | Cosa e` | Uso |
|------|---------|-----|
| `Vec<T>` | Possiede i dati | Quando crei/gestisci la collezione |
| `&mut [T]` | Riferimento mutabile | Quando modifichi senza possedere |
| `&[T]` | Riferimento immutabile | Quando leggi senza modificare |

> **Perche` `&mut [T]` e non `&mut Vec<T>`?**
> Una slice e` piu` flessibile: accetta `Vec`, array, e altre slice.
> `sort_by()` e` definito su `[T]`, non su `Vec<T>`.

---

## 6. `match` su Enum nella Closure

```rust
let ordering = match sort_by {
    SortBy::Name => ...,
    SortBy::Size => ...,
    SortBy::Date => ...,
};
```

Ogni ramo del `match` deve ritornare lo stesso tipo (`Ordering`). Questo e` garantito dal compilatore.

> **Se aggiungessi `SortBy::Type`?**
> Il compilatore ti obbligherebbe ad aggiungerlo qui.
> Zero bug dimenticati!

---

## 7. Combinare Flag

I flag funzionano in combinazione:

```bash
bullo ls --sort size            # Crescente per dimensione
bullo ls --sort size --reverse  # Decrescente per dimensione
bullo ls --long --sort date     # Long + ordinato per data
bullo ls --sort name --reverse  # Z-A
```

---

## 8. Confronto Case-Insensitive

```rust
a.name.to_lowercase().cmp(&b.name.to_lowercase())
```

### Perche` `to_lowercase()`?

Senza, l'ordinamento ASCII mette le maiuscole prima:
```
AGENTS.md  (A = 65 in ASCII)
Cargo.lock (C = 67)
.gitignore (. = 46, prima di tutto!)
```

Con `to_lowercase()`, l'ordinamento e` naturale:
```
.gitignore
.learning
AGENTS.md
Cargo.lock
```

> **Nota:** `.to_lowercase()` crea nuove String. Per grandi directory,
> potresti usare `.cmp()` con una closure che confronta char per char
> in modo case-insensitive, ma per CLI normali `.to_lowercase()` va benissimo.

---

## 9. `SystemTime` e Confronto Temporale

`SystemTime` implementa `Ord`, quindi il confronto e` diretto:

```rust
a.modified_time.cmp(&b.modified_time)
// Vecchio prima ^     ^ Nuovo dopo
```

Con `--reverse`:
```rust
ordering.reverse()
// Nuovo prima ^     ^ Vecchio dopo
```

> **Perche` conservare `SystemTime` e non `DateTime<Local>`?**
> `SystemTime` viene direttamente da `metadata.modified()`.
> La conversione a `DateTime` serve solo per la formattazione.
> Per il confronto, `SystemTime` e` piu` efficiente.

---

## Riepilogo: Combinazioni di Ordinamento

| Comando | Risultato |
|---------|-----------|
| `bullo ls` | Nome A-Z (default) |
| `bullo ls --sort name` | Nome A-Z |
| `bullo ls --sort name --reverse` | Nome Z-A |
| `bullo ls --sort size` | Piccolo → Grande |
| `bullo ls --sort size --reverse` | Grande → Piccolo |
| `bullo ls --sort date` | Vecchio → Nuovo |
| `bullo ls --sort date --reverse` | Nuovo → Vecchio |

---

## Riepilogo Rapido

| Concetto | Keyword/Simbolo | In una riga |
|----------|----------------|-------------|
| Enum CLI | `ValueEnum` | Parsing automatico stringa → enum |
| Closure | `\|a, b\| ...` | Funzione anonima per `sort_by` |
| Ordinamento | `Ordering` | Less, Equal, Greater |
| Invertire | `ordering.reverse()` | Scambia Less ↔ Greater |
| Slice mutabile | `&mut [T]` | Riferimento modificabile |
| Case-insensitive | `.to_lowercase()` | Normalizza per confronto |
| Confronto tempo | `SystemTime::cmp` | Ordine cronologico diretto |
| Dati grezzi | `size_bytes: u64` | Conserva per sorting |
| Dati formattati | `size: String` | Solo per stampa |
| Default value | `default_value = "name"` | Valore predefinito clap |
