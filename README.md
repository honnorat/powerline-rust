# powerline-rust

powerline-rust is an alternative to [powerline-shell](https://github.com/b-ryan/powerline-shell). It's heavily
inspired by it, but focuses on **minimizing execution time**.

> This project started as a fork of [cirho/powerline-rust](https://github.com/cirho/powerline-rust) and has
> since diverged: extra modules (`Jobs`, `Time`, SSH-aware `Host`), `VIRTUAL_ENV_PROMPT` / conda support,
> worktree fixes, and a `gix`-based git backend in place of `libgit2`.

Nobody wants to see latency between pressing enter in their favourite shell and seeing the prompt. That is the
main aim of this tool, and the reason features of other alternatives like dynamic segment selection and
theming via **command-line arguments** are **not possible here**.

Similar results can however be achieved through **customization**.

You have to recompile every time you customize it, but you only change your prompt once in a while — the
performance benefit is worth it.

With default settings `powerline-rust` uses `gix` (gitoxide) for the git prompt. Results vary from system to
system, so if you want every last bit of performance you can try disabling this feature and benchmarking
against the `git` subprocess backend.

## Advantages

- blazing fast (less than 0.010s)
- only necessary dependencies
- runs git backend only when needed (huge time improvements in directories not in git tree)
- optional caching git results in memory or file

## Simple installation

To clone this repository on [Radicle](https://radicle.xyz), simply run:

```bash
rad clone rad://z2FDVfqYkNwYQ3k1uHq3dHSx2xYXm
```

With Git:

```bash
git clone https://github.com/cirho/powerline-rust
cd powerline-rust
# bash shell
cargo install --path .
# zsh shell
cargo install --path . --no-default-features --features=zsh-shell,gitoxide
# fish shell
cargo install --path . --no-default-features --features=bare-shell,gitoxide
```

You can also install one of the examples by adding `--example {name}` to the cargo command.

## Setting up shell

Make sure the executable is in your `$PATH`.

### bash

```bash
function _update_ps1() {
    local __RC=$?;
    PS1="$(NUM_JOBS=$(jobs | grep -v " Done " | wc -l) powerline $__RC)" ;
}

if [ "$TERM" != "linux" ]; then
    PROMPT_COMMAND="_update_ps1; $PROMPT_COMMAND"
fi
```

### zsh

You must also compile with `zsh-shell` feature.

```zsh
_update_ps1() {
    PS1="$(powerline $?)"
}
precmd_functions+=(_update_ps1)
```

### fish

You must also compile with `bare-shell` feature.

```bash
function fish_prompt
    powerline $status
end
```

## Custom shell prompt

Simply create a new Rust program that fulfils your requirements.

```rust
use powerline::{modules::*, theme::SimpleTheme};

fn main() {
    let mut prompt = powerline::Powerline::new();

    prompt.add_module(User::<SimpleTheme>::new());
    prompt.add_module(Host::<SimpleTheme>::new());
    prompt.add_module(Cwd::<SimpleTheme>::new(45, 4, false));
    prompt.add_module(Git::<SimpleTheme>::new());
    prompt.add_module(ReadOnly::<SimpleTheme>::new());
    prompt.add_module(Cmd::<SimpleTheme>::new());

    println!("{}", prompt);
}
```

## Tips and tricks

### Strip executable

Remove unnecessary symbols from the binary to greatly reduce its size. Theoretically it can reduce execution
time.

```bash
cd ~/.cargo/bin/
strip powerline
```

### Use LTO and other

```toml
# Cargo.toml
[profile.release]
lto = true
panic = 'abort'
```

### Target native

Enables optimizations for your specific processor.

```bash
RUSTFLAGS="-C target-cpu=native" cargo ...
```

### Cache untracked files

The Git module can be slower on repos with a large number of untracked files. Read about caching untracked
files [here](https://git-scm.com/docs/git-update-index).

### Custom theme

```rust
use powerline::{modules::*, terminal::Color};

struct Theme;

impl CmdScheme for Theme {
    const CMD_FAILED_BG: Color = Color(161);
    const CMD_FAILED_FG: Color = Color(15);
    const CMD_PASSED_BG: Color = Color(236);
    const CMD_PASSED_FG: Color = Color(15);
}

fn main() {
    let mut prompt = powerline::Powerline::new();
    prompt.add_module(Cmd::<SimpleTheme>::new());

...
```
