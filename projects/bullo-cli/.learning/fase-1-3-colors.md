# Fase 1.3 - Colori nell'Output

> In questa fase aggiungiamo colori all'output di `ls` usando la crate `owo-colors`. Le directory saranno blu, i file eseguibili verdi, i symlink cyan. Imparerai i trait, il metodo chaining e le sequenze ANSI.

---

## 1. `owo-colors` - Colorare il Terminale

`owo-colors` e` una crate moderna per colorare output nel terminale usando **sequenze di escape ANSI**.

```toml
[dependencies]
owo-colors = "4.3"
```

```rust
use owo_colors::OwoColorize;

let text = "Hello";
println!("{}", text.blue());    // Testo blu
println!("{}", text.green());   // Testo verde
println!("{}", text.cyan());    // Testo cyan
println!("{}", text.bold());    // Testo grassetto
println!("{}", text.red().bold()); // Rosso E grassetto (chaining)
```

> **Perche` `owo-colors` e non `colored`?**
> `owo-colors` e` piu` moderna, piu` veloce, e ha un'API piu` pulita.
> Usa il trait `OwoColorize` che si applica a qualsiasi `&str` o `String`.

---

## 2. Trait `OwoColorize`

Il trait `OwoColorize` aggiunge metodi di colorazione a tutti i tipi che lo implementano:

```rust
use owo_colors::OwoColorize;

// Funziona su &str
"directory".blue()

// Funziona su String
String::from("file").green()

// Funziona su qualsiasi tipo che implementa Display
42.red()
```

> **Come fa un trait ad aggiungere metodi a tipi esistenti?**
> Si chiama **trait extension**. Il trait definisce metodi e li implementa
> per tipi standard come `&str`, `String`, `i32`, etc.
> E` lo stesso pattern di `Iterator` che aggiunge `.map()`, `.filter()` a slice e vec.

---

## 3. Method Chaining

I metodi di `owo-colors` possono essere concatenati:

```rust
"text".red().bold().underline()
//      ^^^^  ^^^^^  ^^^^^^^^^
//      rosso  grassetto  sottolineato
```

Ogni metodo ritorna un wrapper che implementa ancora `OwoColorize`, permettendo chaining infinito.

---

## 4. Come Funzionano i Colori nel Terminale

I colori nel terminale usano **sequenze di escape ANSI**:

```
"\x1b[34mtesto blu\x1b[39m"
//  ^^^^^^          ^^^^^^
//  inizia blu      reset colore
```

- `\x1b[` = Escape sequence starter (ESC + `[`)
- `34m` = Colore blu (codice ANSI)
- `39m` = Reset al colore default del terminale

### Codici colore principali

| Codice | Colore | Metodo owo-colors |
|--------|--------|-------------------|
| `30` | Nero | `.black()` |
| `31` | Rosso | `.red()` |
| `32` | Verde | `.green()` |
| `33` | Giallo | `.yellow()` |
| `34` | Blu | `.blue()` |
| `35` | Magenta | `.magenta()` |
| `36` | Cyan | `.cyan()` |
| `37` | Bianco | `.white()` |
| `39` | Default | (reset) |

### Esempio pratico

```rust
// Cosa stampa owo-colors:
"dir".blue()
// => "\x1b[34mdir\x1b[39m"

// Nel terminale vedi: dir (in blu)
// Se fai redirect su file, vedi i codici raw
```

---

## 5. La Funzione `colorize_name`

Abbiamo creato una funzione centralizzata per applicare i colori:

```rust
fn colorize_name(name: &str, file_type: FileType, is_executable: bool) -> String {
    match file_type {
        FileType::Directory => name.blue().to_string(),
        FileType::Symlink => name.cyan().to_string(),
        FileType::File => {
            if is_executable {
                name.green().to_string()
            } else {
                name.to_string()  // Nessun colore
            }
        }
    }
}
```

### Perche` `.to_string()` alla fine?

I metodi di `owo-colors` ritornano un wrapper type (`Fg<Blue, &str>`), non una `String`.
Per stamparlo con `println!` va bene cosi`, ma per calcolare la larghezza delle colonne
serve la stringa pura (senza codici ANSI).

```rust
let colored = "dir".blue();
// colored e` un wrapper, non una String

let s: String = colored.to_string();
// ora s contiene i codici ANSI: "\x1b[34mdir\x1b[39m"
```

---

## 6. Larghezza Colonne con ANSI

**Problema:** i codici ANSI contano come caratteri nella lunghezza della stringa,
ma non occupano spazio visibile nel terminale.

```rust
let colored = "dir".blue().to_string();
colored.len()  // = 14 (con codici ANSI), non 3!
```

**Soluzione:** calcolare le larghezze PRIMA di applicare i colori, usando i nomi raw:

```rust
// Calcola larghezze con nomi raw (senza colori)
let max_name = infos.iter().map(|e| e.name.len()).max().unwrap_or(0);

// Poi applica i colori solo durante la stampa
for info in infos {
    let colored_name = colorize_name(&info.name, info.file_type, false);
    println!("{}", colored_name);
}
```

> **Regola:** separa sempre il calcolo del layout dall'applicazione dei colori.

---

## 7. File Eseguibili

Su Unix, un file e` eseguibile se ha il bit execute impostato:

```rust
#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    // 0o111 = execute bit per owner + group + other
    metadata.permissions().mode() & 0o111 != 0
}
```

### Come funziona `0o111`

```
0o111 = 0o001 | 0o010 | 0o100
      = execute other | execute group | execute owner
```

Se ANY dei tre bit execute e` attivo, il file e` considerato eseguibile.

### Esempi

```
-rwxr-xr-x  -> eseguibile (tutti i bit execute attivi)
-rw-r--r--  -> NON eseguibile (nessun bit execute)
-rwx------  -> eseguibile (solo owner execute)
```

---

## 8. Conditional Compilation per `is_executable`

Windows non ha il concetto di "execute bit", quindi forniamo un fallback:

```rust
#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &fs::Metadata) -> bool {
    false  // Su Windows, nessun file e` "eseguibile" nel senso Unix
}
```

> **Nota:** Su Windows l'eseguibilita` dipende dall'estensione (.exe, .bat, .cmd),
> non dai permessi. Potremmo aggiungere questo controllo in futuro.

---

## 9. Integrazione con `print_listing`

L'unica modifica alle funzioni di stampa e` applicare `colorize_name`:

```rust
// Prima (senza colori):
println!("{}", info.name);

// Dopo (con colori):
let colored_name = colorize_name(&info.name, info.file_type, false);
println!("{}", colored_name);
```

Per il listing lungo, controlliamo anche l'eseguibilita` dai permessi:

```rust
let is_exec = info.permissions.contains('x');
let colored_name = colorize_name(&info.name, info.file_type, is_exec);
```

> **Trucco:** invece di rileggere i metadati, controlliamo se la stringa
> dei permessi contiene `'x'`. Piu` efficiente!

---

## 10. Quando i Colori NON Funzionano

I colori ANSI funzionano solo in terminali che li supportano. Casi comuni:

| Ambiente | Colori | Note |
|----------|--------|------|
| Terminale moderno | ✅ | GNOME Terminal, iTerm2, Windows Terminal |
| `cargo run` | ⚠️ | Dipende dal terminale sottostante |
| Redirect su file | ⚠️ | I codici ANSI vengono scritti nel file |
| CI/CD (GitHub Actions) | ✅ | La maggior parte supporta ANSI |
| Pipe (`|`) | ❌ | Spesso disabilitati automaticamente |

> **Miglioramento futuro:** usare `owo-colors` con feature `supports-colors`
> per rilevare automaticamente se il terminale supporta i colori.

---

## 11. Stili Disponibili in `owo-colors`

Oltre ai colori, `owo-colors` offre stili:

```rust
use owo_colors::OwoColorize;

"text".bold()        // Grassetto
"text".italic()      // Corsivo
"text".underline()   // Sottolineato
"text">.strikethrough() // Barrato
"text">.dimmed()     // Opaco

// Combinazioni
"text".red().bold().underline()
```

### Colori di sfondo

```rust
"text">.on_black()
"text">.on_red()
"text">.on_green()
"text">.on_blue()
```

---

## Riepilogo: Schema Colori

| Tipo File | Colore | Condizione |
|-----------|--------|------------|
| Directory | Blu | Sempre |
| Symlink | Cyan | Sempre |
| File eseguibile | Verde | Bit execute impostato |
| File normale | Nessun colore | Default |

---

## Riepilogo Rapido

| Concetto | Keyword/Simbolo | In una riga |
|----------|----------------|-------------|
| Colorare testo | `.blue()`, `.green()` | Metodi del trait `OwoColorize` |
| Importare trait | `use owo_colors::OwoColorize` | Necessario per i metodi colore |
| Chaining | `.red().bold()` | Combina piu` stili |
| Sequenze ANSI | `\x1b[34m...\x1b[39m` | Codici colore per terminale |
| Convertire wrapper | `.to_string()` | Da wrapper colorato a String |
| Bit execute | `mode & 0o111` | Controlla eseguibilita` Unix |
| Calcolo layout | PRIMA dei colori | I codici ANSI falsano `.len()` |
| Fallback Windows | `#[cfg(not(unix))]` | Niente execute bit su Windows |
