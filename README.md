# rzf

`rzf` is a small Rust fuzzy finder inspired by [`fzf`](https://github.com/junegunn/fzf).

This is a learning project. The goal is to build the core pieces of an interactive
terminal picker from scratch:

- walking a directory tree and collecting candidate file paths
- scoring candidates against a query
- rendering an interactive terminal UI
- moving the selection with the keyboard
- printing the selected path on Enter

## Status

Work in progress.

The fuzzy scoring and directory walking pieces are covered by tests. The terminal
UI is currently being refactored around a small layout type so rendering and input
handling can share the same terminal-size calculations.

## Requirements

- Rust toolchain
- A terminal that supports alternate-screen terminal UIs

## Run

From the project root:

```bash
cargo run
```

`rzf` searches from the current directory:

```bash
cd some/project
cargo run --manifest-path /path/to/rzf/Cargo.toml
```

When a selection is confirmed, the selected path is printed to stdout.

## Controls

| Key | Action |
| --- | --- |
| Type text | Update the fuzzy query |
| Backspace | Delete one query character |
| Up / Ctrl-P | Move selection up |
| Down / Ctrl-N | Move selection down |
| Enter | Confirm current selection |
| Esc / Ctrl-C | Exit without selecting |

## Project Layout

```text
src/
  main.rs  program entry point
  walk.rs  directory traversal
  algo.rs  fuzzy matching and scoring
  ui.rs    terminal UI and interaction loop
```

## Tests

```bash
cargo test
```

The tests focus on matching behavior and directory walking.

## Learning Goals

This project is intentionally small and explicit. The main ideas being practiced
are:

- Rust ownership and borrowing in a terminal application
- separating state, layout, rendering, and input handling
- using `Result` for fallible terminal and filesystem operations
- building up behavior with focused tests
