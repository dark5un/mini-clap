# mini-clap

A minimal CLI argument parser in Rust. Built from scratch through strict TDD with LSP as the compilation guardrail.

No external dependencies. Pure Rust. 11 tests. 86 percent coverage.

## Features

- Positional arguments with name-based retrieval
- Boolean flags with long (--verbose) and short (-v) aliases
- Named options with values (--name Panos)
- Required option validation
- Subcommand support (nested commands with their own args)
- Automatic help text generation
- Error on unknown arguments

## Usage

```rust
use mini_clap::{Command, Arg, Flag, Opt};

let cmd = Command::new("myapp")
    .about("Does something useful")
    .arg(Arg::new("input"))
    .flag(Flag::new("verbose").short('v'))
    .option(Opt::new("name").short('n'));

let matches = cmd.parse(&["file.txt", "--verbose", "--name", "Panos"]).unwrap();
assert_eq!(matches.get("input"), Some("file.txt"));
assert!(matches.get_flag("verbose"));
assert_eq!(matches.get("name"), Some("Panos"));
```

## How it was built

This library was built over 11 TDD cycles, one behavior at a time. Each test was written first, watched fail, then implemented, then watched pass. LSP diagnostics were checked after every write to catch type errors before compilation.

Read the full story in chapter-03.md.

## License

MIT
