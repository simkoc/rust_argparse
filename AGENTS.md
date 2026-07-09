# rust_argparse — Agent Guide

A Python-argparse-inspired command-line argument parser library for Rust.
Version 1.0.0, licensed MIT, published to crates.io as `rust_argparse`.

## Project layout

```
src/
  lib.rs                        # Public API — Parser<F> struct, builder methods, IntoMain trait family
  command_line_parsing_results.rs  # CmdParsingResults (public, non-generic)
  positional_argument.rs        # [positional] argument type
  optional_argument.rs          # -s/--long optional argument type
  flag_argument.rs              # -s/--long boolean flag type
  default_argument.rs           # Pre-set key/value defaults
```

There is no shared trait for argument types (`Help`/`CommandLineParsing` were
removed) — each argument type has plain inherent `help()`/`parse()` methods,
since nothing needed dynamic dispatch over them.

## Key concepts

- **`Parser<F: ?Sized + 'static>`** — the main entry point, generic over `F`, the *exact* function signature (params and return type) invoked for whichever leaf action matches, e.g. `Parser<dyn FnOnce(&CmdParsingResults, Config) -> Output>`. Built with a fluent builder API; each method consumes and returns `Self`. Every parameter the action needs must be named in `F` — nothing is implicitly injected. The whole action tree (including nested sub-actions) shares one `F`.
- **Argument types**
  - *positional* — consumed in declaration order
  - *optional* — `-s`/`--long` with an optional value and default
  - *flag* — `-s`/`--long`, stored as `bool`
  - *default* — key/value pairs injected before parsing starts
- **Actions** — sub-parsers (`add_action`). When registered, the user must supply exactly one matching token; the matching sub-parser then takes over.
- **`with_main`** — attaches the leaf action's function. Accepts closures and named functions directly (no `Box::new`) via the `IntoMain<F>` trait, implemented for two macro-generated families in `lib.rs`: one for plain owned-parameter signatures, one HRTB (`for<'a>`) family specifically for signatures starting with `&CmdParsingResults` (the only reference type this crate hands back). The `#[allow(coherence_leak_check)]` on the HRTB family suppresses a known, harmless rustc coherence-checker false positive (rust-lang/rust#56105) between the two families.
- **`CmdParsingResults`** — plain, non-generic struct holding parsed values keyed by name. Values are `Box<dyn Any>`; callers retrieve them with `get_value::<T>()` and flags with `get_flag()`.
- **`parse` / `parse_cmdline`** — return `Result<(CmdParsingResults, Box<F>), String>`. The boxed function is the matched leaf's `main`, moved out of the parser tree during parsing; call it directly with whatever arguments `F` declares (e.g. `main(&results, config)`).
- **Help** — `--help` / `-h` at any position returns `Err(help_string)`. The caller is responsible for printing and exiting.
- **Parsing order**: defaults → positionals → optionals → flags → actions.

## Build & test

```bash
cargo build          # compile
cargo test           # run unit tests AND README doctests (README is included via #![doc = include_str!(...)] in lib.rs)
cargo clippy         # lint
cargo fmt            # format
cargo package        # verify crate packaging
```

No external dependencies — `[dependencies]` in `Cargo.toml` is empty.

The `README.md` code examples are compiled and executed as part of `cargo test`
(via `#![doc = include_str!("../README.md")]` at the top of `lib.rs`). Any
change to `Parser`'s public API must keep the README examples compiling and
passing — update them in the same change, don't just update `src/`.

## Adding a new argument type

1. Create `src/<type>_argument.rs` following the pattern in `positional_argument.rs` — plain struct with inherent `new`, `help`, and `parse` methods (no shared trait needed).
2. Declare `mod <type>_argument;` in `lib.rs`.
3. Add a `Vec<YourType>` field to `Parser<F>`.
4. Add a builder method `add_<type>(…) -> Parser<F>`.
5. Add a `parse_<type>_arguments` method and call it from `parse_tree` on `Parser<F>` in the correct order.
6. Add unit tests in the `#[cfg(test)]` block at the bottom of the new file.

## Coding Style

- **Fail early, never silently** — return `Err(…)` as soon as invalid state is detected. Do not swallow errors or substitute silent defaults.
- **Function length** — keep functions under 40 lines. If a function grows beyond that, refactor it into smaller, named helpers.
- **Single responsibility** — a function that does more than one thing must be split. Each function should have one clear purpose expressible in its name.
- **Test coverage** — every function requires at minimum three tests: two covering distinct successful behaviors and one covering a failure/error path. Tests live in the `#[cfg(test)]` block in the same file as the code under test.

## Publishing

```bash
cargo publish        # publish to crates.io
```

This crate follows semver. Ensure `version` in `Cargo.toml` is bumped before
publishing — by default bump the patch/revision number (e.g. `1.0.0` →
`1.0.1`); only bump minor or major when the change actually warrants it
(new backwards-compatible functionality → minor, breaking API change →
major).
