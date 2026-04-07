# Fase 0 - Le Fondamenta di Rust

> Questi sono i concetti che hai imparato costruendo la struttura base di Bullo CLI.
> Ogni sezione ha spiegazioni semplici ed esempi presi direttamente dal codice del progetto.

---

## 1. `fn` - Come si definisce una funzione

In Rust le funzioni si dichiarano con la keyword `fn`. Il punto di ingresso di ogni programma è `main()`.

```rust
fn main() {
    println!("Ciao da Bullo CLI!");
}
```

Puoi definire le tue funzioni con parametri e valore di ritorno:

```rust
// Parametro: `path` di tipo Option<String>
// Ritorno:   Result<()>
pub fn execute(path: Option<String>) -> Result<()> {
    // ...
    Ok(())
}
```

> **Nota:** In Rust l'ultima espressione di una funzione è il valore di ritorno
> (senza `;`). `Ok(())` alla fine è il "valore" che la funzione ritorna.

---

## 2. `println!` - Stampare a schermo

Il `!` finale indica che è una **macro**, non una funzione normale.
Le macro vengono "espanse" dal compilatore in codice più complesso.

```rust
// Stampa testo semplice
println!("Hello, world!");

// Usa {} per inserire variabili (richiede il trait Display)
let nome = "Bullo";
println!("Benvenuto in {}", nome);

// Usa {:?} per stampa "tecnica" (richiede il trait Debug)
let path = std::path::PathBuf::from("/tmp");
println!("Path: {:?}", path);  // stampa: Path: "/tmp"

// eprintln! stampa su stderr invece di stdout
eprintln!("Errore: qualcosa è andato storto");
```

---

## 3. Ownership - Il concetto più importante di Rust

In Rust ogni valore ha **un solo proprietario**. Quando il proprietario esce
dallo scope `{}`, il valore viene liberato automaticamente dalla memoria.
Niente garbage collector, niente memory leak.

```rust
fn main() {
    let nome = String::from("Bullo");  // `nome` possiede la stringa

    let altro = nome;                   // ownership TRASFERITA ad `altro`
    //                                  // `nome` non esiste più!

    // println!("{}", nome);            // ERRORE di compilazione
    println!("{}", altro);              // OK
}  // qui `altro` viene liberato automaticamente
```

### Borrowing - "Dare in prestito" senza cedere ownership

Se vuoi che una funzione usi un valore senza prenderlo, usi una **reference** (`&`):

```rust
fn stampa(s: &String) {    // prende in prestito, non possiede
    println!("{}", s);
}

fn main() {
    let nome = String::from("Bullo");
    stampa(&nome);          // passiamo una reference
    println!("{}", nome);   // `nome` è ancora valido qui!
}
```

> **Regola d'oro:** `&T` = "posso guardare ma non modificare".
> `&mut T` = "posso guardare e modificare, ma sono l'unico a farlo".

---

## 4. `String` vs `&str` - Due tipi di stringa

Questo confonde quasi tutti all'inizio. Ecco la distinzione pratica:

| Tipo | Dove vive | Modificabile | Uso tipico |
|------|-----------|--------------|------------|
| `String` | Heap (allocata a runtime) | Sì | Stringhe che costruisci o modifichi |
| `&str` | Binario o stack | No | Testo letterale, riferimenti a String |

```rust
// &str - stringa letterale, nota a compile time, immutabile
let testo: &str = "Bullo CLI";

// String - allocata nell'heap, modificabile
let mut s: String = String::from("Bullo");
s.push_str(" CLI");   // posso aggiungere testo
println!("{}", s);    // "Bullo CLI"

// Da &str a String
let owned = testo.to_string();
let owned2 = String::from(testo);

// Da String a &str (si chiama "deref coercion")
let riferimento: &str = &s;
```

Nel codice di Bullo CLI:
```rust
// In cli.rs - usiamo String perché clap alloca la stringa a runtime
pub struct Ls {
    path: Option<String>,  // String, non &str
}
```

---

## 5. `struct` - Raggruppare dati

Una `struct` è come una "scheda" con campi tipizzati. È la base per creare
tipi personalizzati.

```rust
// Definizione
struct Persona {
    nome: String,
    eta: u32,
}

// Creazione
let p = Persona {
    nome: String::from("Marco"),
    eta: 30,
};

// Accesso ai campi
println!("{} ha {} anni", p.nome, p.eta);
```

Nel progetto, la struct `Cli` definisce l'intera interfaccia a linea di comando:

```rust
// src/cli.rs
#[derive(Parser, Debug)]
#[command(name = "bullo")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,   // l'unico campo: il sotto-comando scelto
}
```

---

## 6. `enum` - Un tipo che può essere "una cosa tra tante"

Gli enum in Rust sono molto più potenti che in altri linguaggi:
ogni variante può contenere **dati diversi**.

```rust
// Enum semplice (come in altri linguaggi)
enum Direzione {
    Nord,
    Sud,
    Est,
    Ovest,
}

// Enum con dati (unico di Rust!)
enum Forma {
    Cerchio(f64),           // contiene il raggio
    Rettangolo(f64, f64),   // contiene larghezza e altezza
    Punto,                  // nessun dato
}
```

Nel progetto usiamo un enum per tutti i comandi disponibili:

```rust
// src/cli.rs
pub enum Commands {
    Ls { path: Option<String> },          // ls può avere un path opzionale
    Cp { source: String, dest: String },  // cp ha sorgente e destinazione
    Mv { source: String, dest: String },
    Rm { path: String },
    // ...
}
```

---

## 7. `match` - Pattern matching esaustivo

`match` è come uno switch/case, ma molto più potente. Il compilatore ti
**obbliga** a gestire tutti i casi possibili. Se dimentichi una variante,
ottieni un errore di compilazione (non un bug a runtime!).

```rust
// src/main.rs - dispatch dei comandi
match cli.command {
    Commands::Ls { path } => {
        // `path` è estratto direttamente dall'enum (destructuring)
        commands::list::execute(path)?;
    }
    Commands::Cp { source, dest } => {
        println!("Copia da {} a {}", source, dest);
    }
    // Se aggiungi una nuova variante a Commands e dimentichi di
    // gestirla qui, il compilatore ti dà errore. Zero bug!
}
```

### `if let` - Match quando interessa solo un caso

```rust
// Equivalente a match con un solo caso che ci interessa
if let Err(e) = run() {
    eprintln!("Errore: {}", e);
    process::exit(1);
}

// È lo stesso di:
match run() {
    Err(e) => {
        eprintln!("Errore: {}", e);
        process::exit(1);
    }
    Ok(()) => {}  // non ci interessa il caso Ok
}
```

---

## 8. `Option<T>` - Gestire l'assenza di un valore

`Option` è come il concetto di "nullable" ma sicuro. Non esiste `null` in Rust:
se un valore può non esserci, si usa `Option`.

```rust
enum Option<T> {
    Some(T),   // c'è un valore di tipo T
    None,      // non c'è nessun valore
}
```

Nel progetto, il path del comando `ls` è opzionale:

```rust
// In cli.rs
Ls { path: Option<String> }

// In list.rs - gestiamo entrambi i casi con match
let target = match path {
    Some(p) => PathBuf::from(p),   // l'utente ha fornito un path
    None    => env::current_dir()?, // usiamo la directory corrente
};
```

Metodi utili su Option:

```rust
let valore: Option<i32> = Some(42);

// unwrap_or - valore di default se None
let n = valore.unwrap_or(0);   // 42

let niente: Option<i32> = None;
let n = niente.unwrap_or(0);   // 0

// map - trasforma il valore interno se presente
let doppio = valore.map(|x| x * 2);  // Some(84)
```

---

## 9. `Result<T, E>` - Gestire gli errori

Rust non ha eccezioni. Ogni funzione che può fallire restituisce `Result`:

```rust
enum Result<T, E> {
    Ok(T),    // successo, contiene il valore T
    Err(E),   // fallimento, contiene l'errore E
}
```

Nel progetto abbiamo definito un **type alias** per non ripetere `BulloError` ovunque:

```rust
// src/error.rs
pub type Result<T> = std::result::Result<T, BulloError>;

// Ora invece di scrivere:
pub fn execute(...) -> std::result::Result<(), BulloError>

// Scriviamo semplicemente:
pub fn execute(...) -> Result<()>
```

### L'operatore `?` - Propagare errori senza boilerplate

```rust
// Senza `?` (verboso):
let dir = match env::current_dir() {
    Ok(d) => d,
    Err(e) => return Err(BulloError::Io(e)),
};

// Con `?` (equivalente, ma conciso):
let dir = env::current_dir()?;
//                          ^ se è Err, ritorna immediatamente l'errore
//                            se è Ok, estrae il valore
```

---

## 10. Il sistema di moduli

Rust organizza il codice in moduli. Ogni file `.rs` è un modulo.

```
src/
├── main.rs          ← crate root (punto di partenza)
├── cli.rs           ← modulo `cli`
├── error.rs         ← modulo `error`
└── commands/        ← modulo `commands` (directory)
    ├── mod.rs       ← "indice" del modulo commands
    └── list.rs      ← sotto-modulo `commands::list`
```

**Regole:**
1. `mod nome;` in main.rs dice "esiste un modulo, cercalo in `src/nome.rs`"
2. Tutto è **privato** per default → usa `pub` per rendere accessibile
3. `use` importa nomi per usarli senza il path completo

```rust
// main.rs
mod cli;        // dichiara il modulo
mod commands;   // dichiara il modulo (cerca src/commands/mod.rs)
mod error;

use cli::{Cli, Commands};   // importa solo i tipi che ci servono
```

```rust
// commands/mod.rs
pub mod list;   // rende il sotto-modulo visibile dall'esterno

// commands/list.rs
use crate::error::Result;   // `crate` = radice del progetto (src/)
```

---

## 11. `#[derive(...)]` - Macro che generano codice automaticamente

Gli attributi `#[derive(...)]` istruiscono il compilatore a generare
automaticamente l'implementazione di certi **trait** (interfacce).

```rust
#[derive(Debug)]        // genera: come stampare con {:?}
#[derive(Clone)]        // genera: come duplicare il valore
#[derive(PartialEq)]    // genera: come confrontare con ==

// Nel progetto:
#[derive(Parser, Debug)]   // Parser = generato da clap per la CLI
pub struct Cli { ... }

#[derive(Debug, thiserror::Error)]  // Error = implementa std::error::Error
pub enum BulloError { ... }
```

---

## 12. Errori custom con `thiserror`

Definire errori chiari rende il codice molto più manutenibile:

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum BulloError {
    // #[from] genera automaticamente la conversione da std::io::Error
    // Questo permette di usare `?` su qualsiasi std::io::Error
    #[error("Errore I/O: {0}")]
    Io(#[from] std::io::Error),

    // {0} si riferisce al primo campo della variante
    #[error("Non trovato: {0}")]
    NotFound(PathBuf),
}
```

Grazie a `#[from]`, l'operatore `?` converte automaticamente gli errori:

```rust
// std::io::Error viene convertito in BulloError::Io automaticamente
let dir = env::current_dir()?;  // Ok!
```

---

## Riepilogo rapido

| Concetto | Keyword/Simbolo | In una riga |
|----------|----------------|-------------|
| Funzione | `fn` | Blocco di codice riutilizzabile |
| Proprietà | ownership | Ogni valore ha un solo proprietario |
| Prestito | `&`, `&mut` | Usa un valore senza possederlo |
| Stringa heap | `String` | Allocata, modificabile |
| Stringa slice | `&str` | Riferimento immutabile a testo |
| Dati strutturati | `struct` | Raggruppa campi tipizzati |
| Tipo o/o | `enum` | Un valore tra varianti possibili |
| Dispatch sicuro | `match` | Gestisce ogni variante, esaustivo |
| Valore opzionale | `Option<T>` | `Some(v)` oppure `None` |
| Risultato fallibile | `Result<T,E>` | `Ok(v)` oppure `Err(e)` |
| Propagazione errore | `?` | Ritorna Err al chiamante |
| Moduli | `mod`, `use`, `pub` | Organizza e isola il codice |
| Codice generato | `#[derive(...)]` | Il compilatore scrive per te |
