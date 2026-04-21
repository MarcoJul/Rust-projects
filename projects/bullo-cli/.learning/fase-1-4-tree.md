# Fase 1.4 - Il Comando `tree`: Visualizzazione ad Albero

> In questa fase implementiamo `bullo tree` con visualizzazione ricorsiva ad albero e flag `--depth N`. Imparerai la ricorsione, il pattern stack di contesto per disegnare alberi, e l'ordinamento intelligente di directory entry.

---

## 1. Ricorsione - Una Funzione che Chiama Se Stessa

La ricorsione e` il modo naturale per attraversare strutture ad albero come le directory:

```rust
fn walk_dir(dir: &Path, depth_limit: u32, current_depth: u32) -> Result<()> {
    // Caso base: fermati se hai raggiunto la profondita` massima
    if current_depth >= depth_limit {
        return Ok(());
    }

    // Leggi la directory corrente
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            // Chiama te stessa per la sotto-directory
            walk_dir(&entry.path(), depth_limit, current_depth + 1)?;
        }
    }

    Ok(())
}
```

### Le Due Regole della Ricorsione

1. **Caso base**: quando fermarsi (altrimenti loop infinito!)
2. **Progresso**: ogni chiamata deve avvicinarsi al caso base

```rust
// Caso base: profondita` massima raggiunta
if current_depth >= depth_limit {
    return Ok(());  // FERMATI
}

// Progresso: current_depth aumenta di 1 ogni chiamata
walk_dir(..., current_depth + 1)?;
```

> **Perche` `current_depth + 1` e non `current_depth++`?**
> Rust non ha `++`. Inoltre, `+ 1` crea un nuovo valore senza mutare
> l'originale. Questo e` piu` sicuro per la ricorsione.

---

## 2. Stack di Contesto - Disegnare l'Albero

Il problema: come disegnare le linee verticali (`│`) correttamente?

```
├── dir1
│   ├── file1     ← questa │ sa che dir1 ha fratelli dopo
│   └── file2     ← questa │ sa che dir1 ha fratelli dopo
└── dir2          ← nessun │ perche` dir2 e` l'ultimo
    └── file3     ← spazio vuoto perche` dir2 e` l'ultimo
```

### La Soluzione: `Vec<bool>`

```rust
let mut parent_prefixes: Vec<bool> = Vec::new();
// true  = questo livello ha ancora fratelli dopo
// false = questo livello e` l'ultimo figlio
```

### Come Funziona

```rust
// Prima di scendere in una sotto-directory:
parent_prefixes.push(!is_last);  // true se ci sono fratelli dopo

// Dopo essere tornati:
parent_prefixes.pop();  // rimuovi il contesto
```

### Costruire il Prefisso

```rust
fn build_prefix(parent_prefixes: &[bool]) -> String {
    let mut prefix = String::new();
    for &has_more in parent_prefixes {
        if has_more {
            prefix.push_str("│   ");  // Linea verticale + spazi
        } else {
            prefix.push_str("    ");  // Solo spazi
        }
    }
    prefix
}
```

> **Pattern push/pop**: e` lo stesso pattern usato per gestire scope
> annidati nei compilatori. Ogni livello aggiunge contesto, poi lo rimuove
> quando torna indietro.

---

## 3. Connettori dell'Albero

Due tipi di connettori:

```rust
let connector = if is_last { "└── " } else { "├── " };
//                                   ^^^^ ultimo figlio
//                                        ^^^^ ha fratelli dopo
```

### Caratteri Unicode

| Carattere | Nome | Uso |
|-----------|------|-----|
| `├` | BOX DRAWINGS LIGHT VERTICAL AND RIGHT | Connettore intermedio |
| `└` | BOX DRAWINGS LIGHT UP AND RIGHT | Connettore finale |
| `─` | BOX DRAWINGS LIGHT HORIZONTAL | Linea orizzontale |
| `│` | BOX DRAWINGS LIGHT VERTICAL | Linea verticale |

> **Perche` Unicode e non ASCII?**
> ASCII usa `+--` e `` `--`` che sono meno leggibili.
> I caratteri box-drawing sono progettati per connettersi perfettamente.

---

## 4. Ordinare: Directory Prima, File Dopo

L'ordinamento predefinito di `ls` mescola file e directory. `tree` mette le directory prima:

```rust
sorted.sort_by(|a, b| {
    let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
    let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);

    match (a_is_dir, b_is_dir) {
        (true, false) => std::cmp::Ordering::Less,     // dir prima di file
        (false, true) => std::cmp::Ordering::Greater,   // file dopo dir
        _ => a.file_name().cmp(&b.file_name()),         // stesso tipo: alfabetico
    }
});
```

### `Ordering` - Il Risultato della Comparazione

```rust
enum Ordering {
    Less,    // a viene prima di b
    Equal,   // a e b sono uguali
    Greater, // a viene dopo di b
}
```

`sort_by()` usa `Ordering` per decidere l'ordine degli elementi.

---

## 5. Contatori Mutabili

Per contare directory e file, usiamo riferimenti mutabili:

```rust
let mut dirs_count = 0u64;
let mut files_count = 0u64;

walk_dir(..., &mut dirs_count, &mut files_count)?;

println!("\n{} directories, {} files", dirs_count, files_count);
```

Nella funzione ricorsiva:

```rust
fn walk_dir(
    ...
    dirs_count: &mut u64,    // riferimento mutabile
    files_count: &mut u64,   // riferimento mutabile
) -> Result<()> {
    if is_dir {
        *dirs_count += 1;  // Deref per modificare il valore
    } else {
        *files_count += 1;
    }
}
```

> **Perche` `*dirs_count` e non `dirs_count`?**
> `dirs_count` e` un `&mut u64` (riferimento).
> `*dirs_count` dereferenzia il riferimento per accedere al valore.
> Senza `*`, staresti cercando di assegnare al riferimento stesso.

---

## 6. Gestire Errori di Lettura

Non tutte le directory sono leggibili (permessi, symlink rotti):

```rust
let entries = match fs::read_dir(dir) {
    Ok(e) => e,
    Err(e) => {
        eprintln!("Errore leggendo '{}': {}", dir.display(), e);
        return Ok(());  // Non propagare l'errore, salta questa dir
    }
};
```

> **Perche` `Ok(())` e non `Err(e)`?**
> Un errore di lettura di una sotto-directory non dovrebbe
> bloccare l'intero albero. Stampiamo un warning e continuiamo.

---

## 7. `filter_map` per Pulire gli Entry

Invece di gestire ogni errore singolarmente:

```rust
let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
//                                ^^^^^^^^^^^^^^^^^^^^^
//                                Scarta entry che hanno errore
```

`filter_map` combina `filter` e `map`:
- `e.ok()` converte `Result<DirEntry>` in `Option<DirEntry>`
- `filter_map` tiene solo i `Some`, scarta i `None`

---

## 8. Il Flag `--depth`

```rust
#[arg(long)]
depth: Option<u32>
```

```bash
bullo tree           # Nessuna limitazione
bullo tree --depth 1 # Solo primo livello
bullo tree --depth 2 # Due livelli
```

### Valore di Default

```rust
let depth_limit = max_depth.unwrap_or(u32::MAX);
// Se --depth non fornito, usa il massimo possibile
```

> **Perche` `u32::MAX` e non `None`?**
> Cosi` il codice della ricorsione e` uniforme:
> controlla sempre `current_depth >= depth_limit`.
> Se `depth_limit` e` `u32::MAX`, praticamente mai vero.

---

## 9. Enumerare con Indice

Per sapere se un entry e` l'ultimo:

```rust
for (i, entry) in sorted.iter().enumerate() {
    let is_last = i == total - 1;
    //             ^^^^^^^^^^^^^^
    //             Ultimo elemento = connettore diverso
}
```

`enumerate()` aggiunge un indice a ogni elemento dell'iteratore:

```rust
["a", "b", "c"].iter().enumerate()
// => (0, "a"), (1, "b"), (2, "c")
```

---

## 10. Testare la Ricorsione

Testare `tree` richiede creare una struttura di directory:

```rust
#[test]
fn test_tree_with_depth_limit() {
    let temp_dir = std::env::temp_dir().join("bullo_tree_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(temp_dir.join("a/b/c")).unwrap();
    std::fs::write(temp_dir.join("a/file.txt"), "test").unwrap();

    execute(Some(temp_dir.clone()), Some(1)).unwrap();

    std::fs::remove_dir_all(&temp_dir).unwrap();
}
```

### `create_dir_all` - Creare Directory Annidate

```rust
std::fs::create_dir_all("a/b/c")
// Crea a/, a/b/, a/b/c/ in una sola chiamata
// Se esistono gia`, non fa nulla (idempotente)
```

---

## 11. Anatomia Completa di `walk_dir`

```
walk_dir(root)
├── leggi root
├── ordina entry (dir prima, poi file)
├── per ogni entry:
│   ├── stampa connettore + nome
│   ├── se directory:
│   │   ├── push contesto
│   │   ├── walk_dir(entry)  ← RICORSIONE
│   │   └── pop contesto
│   └── se file:
│       └── incrementa contatore
└── ritorna Ok(())
```

---

## Riepilogo Rapido

| Concetto | Keyword/Simbolo | In una riga |
|----------|----------------|-------------|
| Ricorsione | funzione chiama se stessa | Attraversa alberi naturalmente |
| Caso base | `if condition { return }` | Previene loop infiniti |
| Stack contesto | `Vec<bool>` push/pop | Tiene traccia dei livelli |
| Connettori | `├──`, `└──`, `│` | Caratteri Unicode box-drawing |
| Ordine | `sort_by` + `Ordering` | Directory prima, file dopo |
| Contatori | `&mut u64`, `*count` | Riferimenti mutabili |
| Enumerare | `.enumerate()` | Aggiunge indice all'iteratore |
| Filtro entry | `.filter_map(\|e\| e.ok())` | Scarta entry con errore |
| Limite profondita` | `unwrap_or(u32::MAX)` | Default illimitato |
| Creare dir | `create_dir_all` | Crea path annidati |
