# Rust Argparse

This crate provides a Python argparse inspired Command Line Parser for Rust.

## Usage

A `Parser<F>` is built by chaining `add_*` calls, then run against a command
line. Each capability below maps to one builder method.

### Positional arguments

`add_positional(name, doc)` consumes the next bare token on the command line,
in declaration order. Use `add_parsed_positional` to convert it from `String`
into another type instead of storing it as-is.

### Optional arguments

`add_optional(name, long, short, default, doc)` reads a `-s value` /
`--long value` pair. If it's absent from the command line, `default` (if
`Some`) is stored instead. `add_parsed_optional` additionally converts the
value from `String`.

### Flags

`add_flag(name, long, short, doc)` reads a `-s` / `--long` switch with no
value; it's `true` if present, `false` otherwise.

### Defaults

`add_default(name, value)` / `add_parsed_default` inject a fixed value into
the results before parsing begins, independent of anything on the command
line — useful for values a sub-action needs that aren't user-supplied.

### Sub-actions

`add_action(parser)` attaches a nested `Parser` as a subcommand: the next
token must match its name, after which the rest of the command line is parsed
by that nested parser. Actions can be nested arbitrarily deep; only leaf
parsers (no further sub-actions) need `with_main`.

### Help

`--help` / `-h` at any position aborts parsing and returns an auto-generated
usage message as the `Err` case of `parse`/`parse_cmdline`.

### Action functions

`Parser<F>` is generic over one type parameter, `F`, which is the *exact*
function signature invoked for whichever leaf action matches the command
line — for example `Parser<dyn FnOnce(Config, Logger) -> ReturnValue>`.
Every parameter your action needs must be named in `F`; there is no implicit
argument. The whole action tree of a single `Parser` (including every nested
sub-action) shares this one signature.

If an action needs the parsed command line values, include
`&CmdParsingResults` as a parameter of `F` — it's the only reference type
this crate ever hands you, and the library supplies it automatically when the
action runs. Anything else in `F` (e.g. `Config`, `Logger`) is supplied by
you, explicitly, when you call the returned function.

`with_main` attaches the function for a leaf `Parser<F>` — a closure or a
named, free-standing function.
`parse` / `parse_cmdline` return `(CmdParsingResults, Box<F>)` on success: the
parsed values, and the function belonging to whichever leaf action matched.
Call that function directly with whatever `F` declares.

## Examples

The code examples are compiled and run as part of `cargo test` (via
`#![doc = include_str!("../README.md")]` in `src/lib.rs`).

### Example 1 — anonymous closure, no parameters at all

```rust
use rust_argparse::Parser;

let parser: Parser<dyn FnOnce() -> Result<(), String>> = Parser::new("greet", "says hello")
    .add_positional("name", "who to greet")
    .with_main(|| Ok(()));

let (results, main) = parser
    .parse(vec!["World".to_string()])
    .expect("parsing should succeed");
assert_eq!(results.get_value::<String>("name"), "World");
main().expect("main should succeed");
```

### Example 2 — named function, results plus two extra parameters

```rust
use rust_argparse::Parser;
use rust_argparse::command_line_parsing_results::CmdParsingResults;

struct Config {
    verbose: bool,
}

struct Logger;

impl Logger {
    fn log(&self, msg: &str) {
        println!("{msg}");
    }
}

type DeployAction = dyn FnOnce(&CmdParsingResults, Config, Logger) -> Result<(), String>;

fn deploy_action(
    results: &CmdParsingResults,
    config: Config,
    logger: Logger,
) -> Result<(), String> {
    let target = results.get_value::<String>("target");
    if config.verbose {
        logger.log(&format!("deploying to {target}"));
    }
    Ok(())
}

let parser: Parser<DeployAction> = Parser::new("deploy", "deploys the app")
    .add_positional("target", "deployment target")
    .with_main(deploy_action);

let (results, main) = parser
    .parse(vec!["production".to_string()])
    .expect("parsing should succeed");
assert_eq!(results.get_value::<String>("target"), "production");

let config = Config { verbose: true };
let logger = Logger;
main(&results, config, logger).expect("main should succeed");
```

### Example 3 — a parser tree with two different leaf actions

Each leaf action gets its own `with_main`, and the whole tree shares one
`Parser<F>` signature. Here each action's return value contains a string
literal hard-coded directly in that action's body, so the value returned by
calling the function identifies which action executed.

```rust
use rust_argparse::Parser;

type ToolAction = dyn FnOnce(&rust_argparse::command_line_parsing_results::CmdParsingResults) -> Result<String, String>;

fn build_parser() -> Parser<ToolAction> {
    Parser::new("tool", "does things")
        .add_action(
            Parser::new("start", "starts the service")
                .add_positional("service", "service name")
                .add_flag("force", "force", 'f', "force start")
                .with_main(|results: &rust_argparse::command_line_parsing_results::CmdParsingResults| {
                    println!(
                        "service={} force={}",
                        results.get_value::<String>("service"),
                        results.get_flag("force")
                    );
                    Ok("start action fired".to_string())
                }),
        )
        .add_action(
            Parser::new("stop", "stops the service")
                .add_positional("service", "service name")
                .add_optional("timeout", "timeout", 't', Some("10"), "shutdown timeout")
                .with_main(|results: &rust_argparse::command_line_parsing_results::CmdParsingResults| {
                    println!(
                        "service={} timeout={}",
                        results.get_value::<String>("service"),
                        results.get_value::<String>("timeout")
                    );
                    Ok("stop action fired".to_string())
                }),
        )
}

// a fresh parser is built per invocation since parsing moves the matched
// leaf's function out of the parser tree
let (start_results, start_main) = build_parser()
    .parse(vec![
        "start".to_string(),
        "web".to_string(),
        "--force".to_string(),
    ])
    .expect("parsing should succeed");
assert_eq!(start_main(&start_results), Ok("start action fired".to_string()));

let (stop_results, stop_main) = build_parser()
    .parse(vec![
        "stop".to_string(),
        "web".to_string(),
        "--timeout".to_string(),
        "30".to_string(),
    ])
    .expect("parsing should succeed");
assert_eq!(stop_main(&stop_results), Ok("stop action fired".to_string()));
```
