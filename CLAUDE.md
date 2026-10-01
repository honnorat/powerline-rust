# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

`powerline-rust` is a fast, statically-configured powerline-style shell prompt generator. It is a fork of
cirho/powerline-rust with extra modules (`jobs`, `time`, `drop`, ssh-aware host, `VIRTUAL_ENV_PROMPT` support,
worktree fixes).

The core design constraint is **execution speed** (<10ms): there is no runtime configuration, no argument
parsing for theming, and no dynamic module selection. Customization happens at compile time — users edit
`src/bin/powerline.rs` (or write their own binary using the library) and rebuild.

## Build and install

The crate is both a library (`src/lib.rs`) and a binary (`src/bin/powerline.rs`). Shell integration is
selected via mutually-exclusive Cargo features.

```bash
# bash (default features = bash-shell + gitoxide)
cargo install --path .

# zsh
cargo install --path . --no-default-features --features=zsh-shell,gitoxide

# fish (raw ANSI escapes)
cargo install --path . --no-default-features --features=bare-shell,gitoxide

# enable the optional Time module
cargo install --path . --features=time
```

Feature flags (`Cargo.toml`):

- `bash-shell` / `zsh-shell` / `bare-shell` — choose ONE; controls escape-sequence wrapping in
  `src/terminal.rs` (`\[…\]` for bash, `%{…%}` for zsh, raw `\x1b[…]` for bare).
- `gitoxide` (default) — link `gix` and use `src/modules/git/gitoxide.rs`; without it the `process` backend
  shells out to `git` (`src/modules/git/process.rs`).
- `time` — enables the `Time` module (uses `libc::strftime`; no extra crate dependency).

Standard cargo workflow otherwise: `cargo build`, `cargo build --release`, `cargo check`, `cargo test`, `cargo
fmt` (config in `rustfmt.toml`: stable options only, `style_edition=2024`, `max_width=120`,
`use_small_heuristics=Max`). MSRV is `rust-version` in `Cargo.toml` (1.88, set by let-chains and `gix`).

The `process` git backend's parser tests only compile without `gitoxide`:
`cargo test --no-default-features --features=bash-shell`.

To try alternate prompt layouts: `cargo run --example minimalistic`.

## Architecture

Rendering is a single linear pass appending styled segments to one `String` buffer (no segment tree).

- `Powerline` (`src/powerline.rs`): a segment's separator is only written when the *next* segment arrives (it
  needs the next background). So **the order of `add_module` calls determines the colors**, and a segment
  cannot be edited once added — style it correctly up front (e.g. `Cwd`'s last component gets a solid
  separator, the others `SEP_THIN`). The trailing separator and `Reset` come from `Display for Powerline`.
- `Module` trait (`src/modules.rs`): `append_segments(&mut self, &mut Powerline)`. A module decides itself to
  emit nothing (e.g. `Git` outside a repo, `Jobs` when `NUM_JOBS=0`).
- Themes are compile-time: `Module<S>` with `S: …Scheme`, a trait of `const Color` items (see `GitScheme`).
  `SimpleTheme` (`src/theme.rs`) implements all schemes; a custom theme is a zero-sized type overriding some.
- `terminal.rs`: `Color(u8)` plus `FgColor`/`BgColor`/`Bold`/`Reset` whose `Display` emits shell-specific
  escapes through the per-feature `OPEN`/`ESC`/`CLOSE` constants. A new shell needs a `#[cfg]` block for them,
  an update of the mutual-exclusion `compile_error!`s, and a check of `Reset` (zsh is special-cased).
- Git: `src/modules/git.rs` aliases one backend (`gitoxide` or `process`) as `internal` via `#[cfg]`. Both
  expose `run_git(&Path) -> GitStats`, given the repo root from `find_git_dir`. New git data goes through
  `GitStats`.

## Shell integration contract

The binary takes the previous command's exit code as `argv[1]` and reads these environment variables:

- `argv[1]` (`$?`): exit code of the previous command. Shown as a segment when non-zero. `Cmd::new()` also
  colors the prompt symbol with it, but the shipped binary uses `Cmd::with_status(true)` and ignores it there.
- `NUM_JOBS`: number of background jobs, shown when non-zero. The shell must set it inline (see the bash
  snippet in `README.md`) because `jobs` is a builtin a child process cannot query.
- `VIRTUAL_ENV_PROMPT`, `VIRTUAL_ENV`, `CONDA_ENV_PATH`, `CONDA_DEFAULT_ENV`: name of the active Python/conda
  environment; the first one set wins, in that order.
- `DROP_ENV`: name of the active `drop run` environment.
- `SSH_CLIENT`, `SSH_TTY`, `SSH_CONNECTION`: detect a remote session. Adds an SSH marker to the host segment,
  and lets `Host`/`User` be shown only when remote (`show_on_remote_shell()`).
- `PWD`: logical working directory, which keeps symlinked paths (unless `resolve_symlinks`).
- `HOME`: abbreviates the home directory to `~` in the path.

When changing what the binary reads, update the shell snippets in `README.md` accordingly.

## Adding a module

1. Create `src/modules/<name>.rs` with a struct `Foo<S> { scheme: PhantomData<S> }`, a `FooScheme` trait of
   `const Color` items, and `impl<S: FooScheme> Module for Foo<S>`.
2. Register the module in `src/modules.rs` (`mod` + `pub use`).
3. Implement `FooScheme for SimpleTheme` in `src/theme.rs`.
4. Add it to the prompt in `src/bin/powerline.rs` at the desired position.
