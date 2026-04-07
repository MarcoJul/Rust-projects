# Agent Guidelines: bullo-cli

## Build, Lint, Test Commands

### Build
```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo check              # Fast syntax/type check
```

### Lint
```bash
cargo clippy             # Lints with clippy
cargo clippy --all-targets --all-features -- -D warnings  # Strict mode
cargo fmt --check        # Check formatting
cargo fmt                # Auto-format code
```

### Test
```bash
cargo test                          # Run all tests
cargo test test_name                # Run specific test
cargo test --lib                    # Unit tests only
cargo test --test integration_test  # Specific integration test
cargo test -- --nocapture           # Show println! output
cargo test -- --test-threads=1      # Run serially
```

### Run
```bash
cargo run                           # Run with debug build
cargo run --release                 # Run with release build
cargo run -- --arg value            # Pass CLI arguments
```

## Code Style Guidelines

### Edition & Toolchain
- **Edition:** 2024
- **Rustc:** 1.93.1 or later
- Use stable Rust unless unstable features are explicitly required

### Imports
```rust
// Order: std → external crates → internal modules
use std::collections::HashMap;
use std::fs;

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::utils;

// Group related imports
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
};
```

### Formatting
- **Indentation:** 4 spaces (enforced by `rustfmt`)
- **Line length:** 100 characters (default rustfmt)
- **Trailing commas:** Required in multi-line expressions
- Run `cargo fmt` before every commit
- Follow rustfmt defaults unless overridden in `rustfmt.toml`

### Types & Annotations
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

### Naming Conventions
- **Functions/variables:** `snake_case`
- **Types/traits:** `PascalCase`
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Modules:** `snake_case` (short, descriptive)
- Avoid abbreviations unless universally understood

### Error Handling
```rust
// Prefer Result over panic
pub fn parse_config(path: &Path) -> Result<Config, ConfigError> { }

// Use ? operator for propagation
let file = File::open(path)?;

// Custom error types with thiserror or similar
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

// Contextual errors with .context() or .map_err()
File::open(path).map_err(|e| MyError::ConfigNotFound(path.clone()))?;
```

### Documentation
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

### Testing
```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() {
        assert_eq!(add(2, 2), 4);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_overflow() {
        add(i32::MAX, 1);
    }
}

// Integration tests in tests/ directory
// Use descriptive test names: test_<scenario>_<expected_outcome>
```

### Common Patterns
- **Constructors:** Use `new()` or `from_*()` methods
- **Builders:** Consider the builder pattern for complex initialization
- **Options:** Use `Option<T>` instead of nullable pointers
- **Lifetimes:** Keep explicit lifetimes minimal; let the compiler infer
- **Trait bounds:** Prefer `impl Trait` for simple cases, generics for complex

### Avoid
- Unnecessary clones (use borrowing)
- Unwrap/expect in library code (use `?` and `Result`)
- Overly complex type signatures (refactor to type aliases)
- Global mutable state (use interior mutability patterns)

## Workflow

1. **Before coding:** Run `cargo check` to verify compilation
2. **During coding:** Run `cargo clippy` to catch common mistakes
3. **Before commit:** 
   - `cargo fmt`
   - `cargo test`
   - `cargo clippy --all-targets -- -D warnings`
4. **Commit messages:** Concise, imperative mood: "Add feature X" not "Added feature X"

## Dependencies

When adding dependencies:
```bash
cargo add <crate>           # Add dependency
cargo add --dev <crate>     # Add dev dependency
cargo update                # Update dependencies
```

Choose well-maintained crates with recent updates and good documentation.
