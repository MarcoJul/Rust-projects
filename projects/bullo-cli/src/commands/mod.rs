// =============================================================================
// commands/mod.rs - Indice del modulo commands
// =============================================================================
//
// CONCETTI RUST IN QUESTO FILE:
//
// 1. `mod list;` - dichiara il sotto-modulo `list`, che Rust cercherà
//    nel file `src/commands/list.rs`.
//
// 2. `pub mod` vs `mod`:
//    - `pub mod list;` = il modulo e` visibile FUORI da `commands`
//      (main.rs puo` fare `commands::list::qualcosa`)
//    - `mod list;` = il modulo e` privato, solo `commands` puo` usarlo
//
// 3. `pub use` (re-export):
//    Permette di "riesportare" nomi da sotto-moduli, cosi` chi importa
//    `commands` non deve conoscere la struttura interna.
//    Esempio: `commands::execute` invece di `commands::list::execute_ls`
//
// Per ora abbiamo solo `list`, ma aggiungeremo gli altri nelle fasi successive.
// =============================================================================

// Sotto-moduli (uno per comando)
pub mod list;

// Qui in futuro aggiungeremo:
// pub mod copy;
// pub mod move_cmd;   // `move` e` una keyword Rust, non possiamo usarla!
// pub mod remove;
// pub mod preview;
// pub mod open;
