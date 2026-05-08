---
project_name: 'bullo-cli'
user_name: 'christian'
date: '2026-05-08'
sections_completed:
  ['technology_stack', 'language_rules', 'testing_rules', 'framework_rules', 'critical_rules', 'code_quality', 'dependencies']
status: 'complete'
rule_count: 25
optimized_for_llm: true
---

# Project Context for AI Agents

_Questo file contiene regole critical e pattern che gli agenti AI devono seguire quando implementano codice in questo progetto. Focus su dettagli non ovvi che gli agenti potrebbero altrimenti ignorare._

---

## Technology Stack & Versions

**Core Technologies:**
- **Rust** — 2024 edition, rustc 1.93.1+
- **bullo-cli** — File manager CLI

**Key Dependencies:**
- `chrono` 0.4.44 — datetime handling
- `clap` 4.5.60 — CLI parsing con derive
- `owo-colors` 4.3.0 — terminal colors
- `thiserror` 2.0.18 — error handling
- `uzers` 0.12.2 — Unix user/group lookup

**Build Commands:**
```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo check              # Fast syntax/type check
```

**Lint Commands:**
```bash
cargo clippy             # Lints with clippy
cargo clippy --all-targets --all-features -- -D warnings  # Strict mode
cargo fmt --check        # Check formatting
cargo fmt                # Auto-format code
```

**Test Commands:**
```bash
cargo test                          # Run all tests
cargo test test_name                # Run specific test
cargo test --lib                    # Unit tests only
cargo test -- --nocapture           # Show println! output
cargo test -- --test-threads=1      # Run serially
```

**Run Commands:**
```bash
cargo run                           # Run with debug build
cargo run --release                 # Run with release build
cargo run -- --arg value            # Pass CLI arguments
```

---

## Critical Implementation Rules

### Language-Specific Rules (Rust)

**Imports Ordering:**
```rust
// Order: std → external crates → internal modules
use std::collections::HashMap;
use std::fs;

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::utils;
```

**Formatting:**
- **Indentation:** 4 spaces (enforced by `rustfmt`)
- **Line length:** 100 characters (default rustfmt)
- **Trailing commas:** Required in multi-line expressions
- Run `cargo fmt` before every commit

**Types & Annotations:**
```rust
// Explicit types for public APIs
pub fn process_data(input: &str) -> Result<Vec<Data>, Error> { }

// Type inference acceptable for locals
let count = items.len();

// Prefer owned types in structs unless borrowing is essential
pub struct Config {
    pub name: String,      // Not &'a str
    pub values: Vec<i32>,
}

// Use type aliases for complex types
type Result<T> = std::result::Result<T, MyError>;
```

**Naming Conventions:**
- **Functions/variables:** `snake_case`
- **Types/traits:** `PascalCase`
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Modules:** `snake_case` (short, descriptive)
- Avoid abbreviations unless universally understood

**Error Handling:**
```rust
// Prefer Result over panic
pub fn parse_config(path: &Path) -> Result<Config, ConfigError> { }

// Use ? operator for propagation
let file = File::open(path)?;

// Custom error types with thiserror
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}
```

### Testing Rules

**Test Structure:**
```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() {
        assert_eq!(add(2, 2), 4);
    }
}

// Integration tests in tests/ directory
// Use descriptive test names: test_<scenario>_<expected_outcome>
```

**Test Organization:**
- Unit tests live in `#[cfg(test)] mod tests` within the source file
- Integration tests in `tests/` directory at project root
- Test naming: `test_<scenario>_<expected_outcome>`

### Critical Don't-Miss Rules

**Avoid:**
- Unnecessary clones (use borrowing)
- `unwrap()`/`expect()` in library code (use `?` and `Result`)
- Overly complex type signatures (refactor to type aliases)
- Global mutable state (use interior mutability patterns)

**Platform-Specific Code:**
```rust
// Unix-specific code
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

// Windows fallback
#[cfg(not(unix))]
fn format_permissions(metadata: &fs::Metadata) -> String {
    // ...
}
```

**Documentation Requirements:**
```rust
/// Brief one-line summary.
///
/// Detailed explanation if needed. Use markdown.
///
/// # Examples
/// ```
/// let result = my_function(42);
/// assert_eq!(result, 84);
/// ```
///
/// # Errors
/// Returns `Err` if the input is negative.
pub fn my_function(x: i32) -> Result<i32, MyError> { }
```

---

## Code Quality & Style Rules

**Before coding:** Run `cargo check` to verify compilation
**During coding:** Run `cargo clippy` to catch common mistakes
**Before commit:**
1. `cargo fmt`
2. `cargo test`
3. `cargo clippy --all-targets -- -D warnings`

**Commit messages:** Concise, imperative mood: "Add feature X" not "Added feature X"

---

## Dependencies

When adding dependencies:
```bash
cargo add <crate>           # Add dependency
cargo add --dev <crate>     # Add dev dependency
cargo update                # Update dependencies
```

Choose well-maintained crates with recent updates and good documentation.

---

## Usage Guidelines

**For AI Agents:**

- Read this file before implementing any code
- Follow ALL rules exactly as documented
- When in doubt, prefer the more restrictive option
- Update this file if new patterns emerge

**For Humans:**

- Keep this file lean and focused on agent needs
- Update when technology stack changes
- Review quarterly for outdated rules
- Remove rules that become obvious over time

_Last Updated: 2026-05-08_