# àṣẹ ("ah-sheh")

àṣẹ (pronounced roughly “ah‑sheh”) is a small Unix-style interactive shell written in Rust.

It’s designed to feel familiar if you live in `bash`/`zsh`, but with a compact, readable implementation that’s easy to explore and hack on.

---

## Features

- **Interactive shell**
  - Colorful prompt showing the shell name and current directory.
  - Shows the current Git branch when you’re in a Git repo (or a child directory).
  - Multi-line input for unclosed quotes.

- **Builtins**
  - `cd` – change directory (`cd`, `cd ~`, `cd /path`).
  - `pwd` – print the current directory.
  - `echo` – print arguments.
  - `exit [CODE]` – exit the shell with an optional status.
  - `type NAME...` – tell you whether a name is a builtin or which executable it resolves to.
  - `history [N]` – list recent commands (optionally last `N`), with redirect support.

- **External commands**
  - Resolves executables via `PATH`, just like a typical Unix shell.
  - You can run `ls`, `grep`, `git`, etc. as usual.

- **Redirection**
  - `>` / `1>` – overwrite stdout.
  - `>>` / `1>>` – append stdout.
  - `2>` / `2>>` – redirect stderr and append stderr.
  - Redirection targets support `~` and `$VAR` (e.g. `> ~/out.txt`, `2> $LOGFILE`).

- **Pipelines**
  - `cmd1 | cmd2 | cmd3` using OS pipes between external programs.
  - Last stage honours stdout/stderr redirections.

- **Control operators**
  - `;` – run commands in sequence, regardless of the previous command’s exit status.
  - `&&` – run the next command only if the previous one succeeded (exit status 0).
  - `||` – run the next command only if the previous one failed (exit status ≠ 0).
  - Splitting respects quotes (e.g. `echo "hello; world"` is one command).

- **History**
  - Up/down arrows navigate previous commands.
  - `history` and `history N` print numbered history.
  - Output can be redirected like any other command.

- **Tab completion**
  - First word: builtins + executables found in `PATH`.
  - `cd` arguments: filesystem path completion.

- **Basic expansions**
  - `$VAR` – environment variable expansion.
  - `~` / `~/...` – tilde expansion using `$HOME`.
  - Globbing: `*`, `?`, `[...]` in arguments expand to matching paths.

This is intentionally _not_ a full POSIX shell; it’s a small, usable shell you can read in an afternoon.

---

## Using àṣẹ

### Prompt

The prompt looks like:

```text
àṣẹ [my-project (main)] >
```

- `my-project` is the current directory name.
- `(main)` is the current Git branch when inside a repo (omitted otherwise).

### Builtins in a bit more detail

- **`cd [DIR]`**
  - `cd` or `cd ~` → `$HOME`.
  - `cd /tmp`, `cd relative/path` as you’d expect.

- **`pwd`**
  - Prints the current working directory.

- **`echo ARG...`**
  - Writes arguments joined by a space, followed by a newline.

- **`exit [CODE]`**
  - `exit` exits with status `0`.
  - `exit 1` exits with status `1`, etc.

- **`type NAME...`**
  - For each `NAME`, prints either:
    - `NAME is a shell builtin`, or
    - `NAME is /full/path/to/executable`, or
    - `NAME: not found`.

- **`history [N]`**
  - `history` prints all commands in the in-memory history.
  - `history 20` prints only the last 20.
  - Output can be redirected:
    ```bash
    history > hist.txt
    history 50 >> last50.txt
    ```

### Pipelines and redirection examples

```bash
ls | wc -l
ps aux | grep ase | wc -l
cat Cargo.toml | sed -n '1,5p'
```

```bash
echo hello > out.txt
echo more >> out.txt
ls not-a-dir 2> errors.log
```

### Control operator examples

```bash
echo one ; echo two
# Prints "one" then "two", always runs both.

mkdir foo && cd foo
# Creates foo, then changes into it only if mkdir succeeded.

false || echo "previous command failed"
# Runs echo only because false exits with non-zero.
```

---

## Installation

### macOS

**Homebrew** (recommended):

```bash
brew install steph-crown/tap/ase-shell
```

**Shell installer (macOS or Linux):**

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/steph-crown/ase/releases/download/v0.2.3/ase-shell-installer.sh | sh
```

This downloads the appropriate binary for your platform and puts `ase` on your `PATH`.

### Linux

**Shell installer (most distros):**

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/steph-crown/ase/releases/download/v0.2.3/ase-shell-installer.sh | sh
```

Alternatively, grab the tarball for your architecture from the
[releases page](https://github.com/steph-crown/ase/releases), unpack it,
and move the `ase` binary somewhere on your `PATH` (e.g. `/usr/local/bin`).

### Windows

**PowerShell installer:**

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/steph-crown/ase/releases/download/v0.2.3/ase-shell-installer.ps1 | iex"
```

You can also download the `.zip` or `.msi` for your architecture from the
[releases page](https://github.com/steph-crown/ase/releases) and install it manually.

### OS‑agnostic options

#### Cargo

You’ll need a recent Rust toolchain (`rustup` recommended):

```bash
cargo install ase-shell
```

This installs the `ase` binary into `~/.cargo/bin`. Make sure that directory is on your `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then run:

```bash
ase
```

#### npm

For a global CLI (if you prefer managing it via npm):

```bash
npm install -g ase-shell
ase
```

### From source

```bash
git clone https://github.com/steph-crown/ase.git
cd ase
cargo build --release
```

The binary will be at:

```bash
target/release/ase
```

You can copy or symlink it somewhere on your `PATH`, for example:

```bash
sudo cp target/release/ase /usr/local/bin/ase
```

---

## What’s not implemented (yet)

This is **not** a drop-in replacement for `bash`/`zsh`. Notable gaps:

- No job control (`&`, `jobs`, `fg`, `bg`, `Ctrl+Z`).
- No shell scripting language (no `if`, `for`, `while`, functions, `$@`, etc.).
- No here-docs (`<<`), here-strings, or advanced redirections (`2>&1`, arbitrary FDs).
- No history expansion (`!!`, `!123`, `!foo`).

I plan to add some of these in future releases, so watch out for them.

---

## Contributing

Issues and pull requests are welcome. The codebase is intentionally small and straightforward:

- `src/main.rs` – REPL loop, prompt handling, history wiring.
- `src/lib.rs` – crate exports and constants.
- `src/commands.rs` – `Cmd` enum, builtins, external command handling, pipelines.
- `src/commands/parse.rs` – token parsing, redirections, expansions.
- `src/commands/targets.rs` – stdout/stderr target types and writer helpers.
- `src/repl.rs` – Rustyline setup and completion.
- `src/utils.rs` – prompt and `pwd` helpers.

If you build something cool on top or find rough edges in the UX, feel free to open an issue or PR.

---

## License

Dual-licensed under either:

- MIT license
- Apache License, Version 2.0

at your option.
