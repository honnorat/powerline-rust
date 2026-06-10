# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

`powerline-rust` is a fast, statically-configured powerline-style shell prompt generator. It is a fork
(`version = "0.3.1-mh01"`) of cirho/powerline-rust with extra modules (`jobs`, `time`, ssh-aware host,
`VIRTUAL_ENV_PROMPT` support, worktree fixes).

The core design constraint is **execution speed** (<10ms): there is no runtime configuration, no argument
parsing for theming, and no dynamic module selection. Customization happens at compile time — users edit
`src/bin/powerline.rs` (or write their own binary using the library) and rebuild.

## Build and install

The crate is both a library (`src/lib.rs`) and a binary (`src/bin/powerline.rs`). Shell integration is
selected via mutually-exclusive Cargo features.

```bash
# bash (default features = bash-shell + libgit)
cargo install --path .

# zsh
cargo install --path . --no-default-features --features=zsh-shell,libgit

# fish (raw ANSI escapes)
cargo install --path . --no-default-features --features=bare-shell,libgit

# enable the optional Time module
cargo install --path . --features=time
```

Feature flags (`Cargo.toml`):

- `bash-shell` / `zsh-shell` / `bare-shell` — choose ONE; controls escape-sequence wrapping in
  `src/terminal.rs` (`\[…\]` for bash, `%{…%}` for zsh, raw `\x1b[…]` for bare).
- `libgit` (default) — link `git2` and use `src/modules/git/libgit.rs`; without it the `process` backend shells out to `git` (`src/modules/git/process.rs`).
- `time` — enables the `Time` module via `chrono`.

Standard cargo workflow otherwise: `cargo build`, `cargo build --release`, `cargo check`, `cargo test`, `cargo
fmt` (config in `rustfmt.toml`: `max_width=130`, `use_small_heuristics=Max`).

To try alternate prompt layouts: `cargo run --example minimalistic`.

## Architecture

Rendering is a single linear pass that appends styled segments to a `String` buffer; there is no intermediate
segment tree.

- `Powerline` (`src/powerline.rs`) owns the buffer and the `last_style` from the previously written segment.
  `write_segment` emits the next segment's background, the previous segment's separator (in the previous bg →
  next bg transition color), then the foreground and content. The trailing separator and `Reset` are emitted
  by `Display for Powerline`. **Order of `add_module` calls determines visual order**, and each segment's
  separator color depends on the next segment — so reordering changes the rendered colors.
- `Module` trait (`src/modules.rs`) — every module implements `append_segments(&mut self, &mut Powerline)`.
  Modules decide internally whether to emit nothing (e.g. `Git` returns early if `find_git_dir` fails; `Jobs`
  skips if `NUM_JOBS=0`).
- Themes are **compile-time generic parameters**, not runtime values. Each module is `Module<S>` where `S`
  implements a per-module `…Scheme` trait of `const Color` associated constants (see `GitScheme` in
  `src/modules/git.rs`). `SimpleTheme` in `src/theme.rs` implements every scheme; custom themes are new
  zero-sized types implementing the schemes you want to override. Modules carry `PhantomData<S>` — there is no
  theme object at runtime.
- `terminal.rs` defines `Color(u8)` (256-color palette indices) and the `FgColor`/`BgColor`/`Reset` newtypes
  whose `Display` impls produce shell-specific escape sequences gated on the shell feature flag. Adding a new
  shell means adding a `#[cfg(feature = "…")]` branch in all three `Display` impls.
- Git backend selection happens via `#[cfg]` aliasing inside `src/modules/git.rs`: both `libgit` and `process`
  submodules expose `run_git(&Path) -> GitStats`, and the parent re-aliases one as `internal`. New git data
  flows through `GitStats`.

## Shell integration contract

The binary expects the previous command's exit code as `argv[1]` and reads these env vars:

- `$?` (positional arg) — used by `Cmd` (color of the prompt symbol) and `ExitCode` (numeric segment when
  non-zero).
- `NUM_JOBS` — read by `Jobs`; the bash snippet in `README.md` sets it inline because `jobs` is a shell
  builtin that can't be queried from a child process.
- `VIRTUAL_ENV_PROMPT` (preferred) / `VIRTUAL_ENV` — read by `VirtualEnv`.
- `SSH_CLIENT` / `SSH_TTY` — `Host` switches to SSH colors when set.

When changing what the binary reads, update the shell snippets in `README.md` accordingly.

## Adding a module

1. Create `src/modules/<name>.rs` with a struct `Foo<S> { scheme: PhantomData<S> }`, a `FooScheme` trait of
   `const Color` items, and `impl<S: FooScheme> Module for Foo<S>`.
2. Register the module in `src/modules.rs` (`mod` + `pub use`).
3. Implement `FooScheme for SimpleTheme` in `src/theme.rs`.
4. Add it to the prompt in `src/bin/powerline.rs` at the desired position.
