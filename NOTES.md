# rzf learning notes

Project: `/home/omae/personal/rzf`

Goal: build a small Rust fuzzy finder inspired by `fzf`.

This file is for study. It should guide the next session and explain why the
code is shaped a certain way. It is not meant to be polished product
documentation.

## Current state

The project currently has four main modules:

```text
src/main.rs   program entry point, CLI mode, candidate loading
src/walk.rs   default filesystem candidate discovery
src/fuzzy.rs  fuzzy matching and scoring
src/ui.rs     terminal UI, state, layout, rendering, event loop
```

Verified checks:

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

All three currently pass.

## Current behavior

Interactive mode:

```bash
rzf
```

If stdin is a terminal, `rzf` walks the current directory and uses those files as
candidates.

Piped input mode:

```bash
rg --files | rzf
```

If stdin is piped, `rzf` reads one candidate per input line.

Filter mode:

```bash
rzf --filter main
rg --files | rzf --filter main
```

`--filter` ranks candidates non-interactively and prints matching paths.

Interactive selection:

```text
Enter       return Some(path)
Esc/Ctrl-C  return None
```

The selected path is printed by `main.rs` after `ui::run(...)` returns.

## Why stdin support matters

`fzf` is mainly a filter over a list of candidates.

This is the core model:

```bash
some-command-that-prints-lines | fzf
```

The default file listing is only a convenience. That means `rzf` should keep
these ideas separate:

```text
candidate source   where lines come from
matching/ranking   how query text filters and orders those lines
UI                 how the user edits the query and chooses a result
```

This is why `main.rs` has candidate loading logic that is reused by both
interactive mode and `--filter`.

## Terminal UI concept

The terminal is a grid:

```text
x: 0 1 2 3 4 ...
y: 0  first row
y: 1  second row
y: 2  third row
```

`cursor::MoveTo(x, y)` moves the terminal cursor to a grid cell.

`Print(...)` writes text at the current cursor position.

So the order matters:

```text
move first
print text
move real cursor where typing should continue
```

For the prompt, the concept is:

```rust
let prompt_row = height - 1;
let cursor_x = 2 + state.query.len() as u16;

queue!(
    out,
    cursor::MoveTo(0, prompt_row),
    Print(format!("> {}", state.query)),
    cursor::MoveTo(cursor_x, prompt_row),
)?;
```

Why `2`?

`"> "` is two characters before the query:

```text
> abc
01 234
```

So if the query is `abc`, the cursor should be at:

```rust
2 + 3 = 5
```

Later, for Unicode text, use:

```rust
state.query.chars().count()
```

instead of:

```rust
state.query.len()
```

because `.len()` counts bytes, not displayed characters.

## Why `Layout` exists

The UI has values that come from the terminal size:

```text
prompt_row     where the query prompt is drawn
size_row       where the count/separator row is drawn
max_result     how many results can be visible
width          how wide the terminal is
```

These are not only rendering details.

The event loop also needs `max_result` so it can keep the selected item visible
after moving up or down.

That is why `ui.rs` has a layout type:

```text
run:
  controls lifecycle and event loop

Layout:
  converts terminal size into row positions and visible result count

render:
  draws using State + Layout

State:
  owns query, results, selected index, and scroll offset
```

The important design idea is ownership of arithmetic:

```text
Layout should own layout arithmetic.
Callers should use the resulting fields.
```

That keeps `run(...)` from duplicating row math that belongs to rendering.

## Selection and scrolling

`State` has:

```text
selected       selected index in the full results list
scroll_offset  first visible result index
```

The useful invariant is:

```text
scroll_offset <= selected < scroll_offset + visible_count
```

After moving selection, call:

```rust
state.keep_selected_visible(visible_count);
```

When the query changes, reset:

```text
selected = 0
scroll_offset = 0
```

Why reset on query changes?

Changing the query creates a new ranked result list. The old selected index may
refer to a different item or may be out of range, so the simplest correct
behavior is to return to the top.

## Fuzzy scoring

`fuzzy::score(query, candidate)` returns:

```rust
Option<i32>
```

Meaning:

```text
Some(score)  query is a subsequence of candidate
None         query does not match candidate
```

The current scoring rewards:

```text
base match        each matched query character
first char        match at candidate position 0
consecutive       adjacent matched characters
boundary          match after _, -, /, ., or space
camel case        uppercase match after lowercase
```

The current scoring penalizes:

```text
gap start         first skipped candidate character
gap extend        later skipped candidate characters
```

Current intentional simplification:

```text
matching is case-sensitive
```

Real `fzf` has smarter case behavior. That can be a later study task.

## File walking

`walk.rs` is the default candidate source for interactive terminal use.

Candidates used to look like:

```text
./Cargo.toml
./src/main.rs
```

The walker now strips the leading `./` so they display as:

```text
Cargo.toml
src/main.rs
```

The key code shape is:

```rust
let file = file.strip_prefix("./").unwrap_or(file);
files.push(file.to_string());
```

`strip_prefix("./")` returns `Some(...)` if the prefix exists, otherwise `None`.

`unwrap_or(file)` means: use the stripped path if it exists, otherwise keep the
original path.

Design note:

Skipping directories such as `.git` or `target` belongs to the default file
listing convenience mode, not to fuzzy matching. If candidates come from stdin,
`rzf` should filter exactly the candidate lines it receives.

## Next study tasks

### 1. Rename unclear fields

Current names:

```text
max_quantity
max_result
```

Clearer names:

```text
total_candidates
visible_count
```

Why:

The new names describe what the values mean in the UI instead of how they were
first calculated.

### 2. Make rendering read-only

Current shape:

```rust
fn render(state: &mut State, layout: &Layout, out: &mut Stdout) -> io::Result<()>
```

Better shape:

```rust
fn render(state: &State, layout: &Layout, out: &mut Stdout) -> io::Result<()>
```

Why:

Rendering should draw state, not mutate it. A read-only reference makes that
rule visible in the function signature.

### 3. Extract shared ranking logic

Current duplication:

```text
ui.rs    State::rerank ranks candidates
main.rs  filter(...) ranks candidates again
```

Preferred shape:

```text
rank(query, candidates) -> Vec<(score, candidate)>
```

Why:

One ranking function makes interactive mode and `--filter` use the same behavior.

### 4. Decide default skipped directories

Current walker skips `.git`.

For this Rust learning project, skipping `target` in the default walker is
probably useful because build artifacts can be noisy and can make filesystem
tests less deterministic.

Important distinction:

```text
default walker policy != fuzzy finder policy
```

If a user pipes `target/debug/rzf` into stdin, `rzf` should still be allowed to
match it.

### 5. Add tests when behavior changes

Good tests to add or adjust:

```text
walk.rs    skipped default directories
fuzzy.rs   shared ranking order after extraction
main.rs    candidate lines from stdin parsing
ui.rs      pure state movement helpers, if made public/testable
```

## Resume commands

```bash
cd /home/omae/personal/rzf
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```
