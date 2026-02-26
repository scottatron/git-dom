# git-dom 🏠

A an opinionated & friendlier UX for git submodules.

Git submodules are powerful but the CLI is clunky. `git-dom` wraps existing submodule commands with simpler, more intuitive operations — like a good parent managing unruly children. 😏

## Installation

### From GitHub Releases

Download a prebuilt binary from [Releases](https://github.com/scottatron/git-dom/releases) and place it on your `$PATH`:

```sh
# Example for macOS (Apple Silicon)
tar xzf git-dom-aarch64-apple-darwin.tar.gz
mv git-dom ~/.local/bin/
```

### From source

```sh
cargo install --git https://github.com/scottatron/git-dom.git
```

### Verify

```sh
git dom -h
```

> **Note:** `git dom --help` uses `git help dom`, so it requires a `git-dom` man page in your `MANPATH`. Install once with `git dom man --install`.

## Commands

### `git dom ls`

List all submodules at a glance with branch, commit, and dirty status.

```
my-lib  a1b2c3d  main  clean
utils   e4f5g6h  main  dirty
```

### `git dom status [name]`

Rich per-submodule status — like `git status` but for every submodule:

- Current branch and HEAD commit
- Ahead/behind upstream
- Staged, modified, and untracked files
- Pending changes in the parent repo

### `git dom clone <url>`

Add a submodule with a Go-style path convention:

```sh
git dom clone tokio-rs/tokio
# → src/github.com/tokio-rs/tokio

git dom clone github.com/tokio-rs/tokio
# → src/github.com/tokio-rs/tokio
```

`owner/repo` defaults to GitHub.

Prompts to commit when running interactively. Use `--no-commit` to skip.

The root path (`src/` by default) is configurable:

```sh
git config dom.root vendor/
```

Full URLs also work:

```sh
git dom clone https://github.com/user/repo.git
git dom clone git@github.com:user/repo.git
```

### `git dom pull [name]`

Fetch and update all submodules (or a specific one) from upstream, then handle the commit:

```sh
git dom pull              # update all
git dom pull my-lib       # update just my-lib
git dom pull --commit=prompt  # ask before committing
```

Commit behaviour is configurable via `git config dom.commit` or `--commit`:

| Mode     | Behaviour                            |
|----------|--------------------------------------|
| `auto`   | Commit immediately (default)         |
| `stage`  | Stage changes, don't commit          |
| `prompt` | Show diff and ask before committing  |

### `git dom rm <name>`

Remove a submodule cleanly in one step — no more manually editing `.gitmodules`, `.git/config`, and removing the worktree:

```sh
git dom rm my-lib
```

Prompts to commit when running interactively. Use `--no-commit` to skip.

### `git dom diff [name]`

Show changes across submodules — updated refs, dirty working trees:

```sh
git dom diff          # summary for all
git dom diff --full   # full diffs within each submodule
```

### `git dom foreach <command>`

Run a command in every submodule:

```sh
git dom foreach git fetch origin
git dom foreach --parallel cargo check
```

### `git dom man`

Generate or install the `git-dom.1` man page:

```sh
git dom man --install
# installs to $XDG_DATA_HOME/man/man1/git-dom.1
# (or ~/.local/share/man/man1/git-dom.1 if XDG_DATA_HOME is unset)

git dom --help
```

You can also write to a custom path:

```sh
git dom man --output ./git-dom.1
```

## Shell Completions

Generate and install completions for your shell. Submodule names are completed dynamically.

### Zsh

```sh
mkdir -p ~/.zfunc
git-dom completions zsh > ~/.zfunc/_git-dom
```

Add to `.zshrc` (before `compinit`):

```sh
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

### Bash / Fish / PowerShell

```sh
git-dom completions bash > ~/.local/share/bash-completion/completions/git-dom
git-dom completions fish > ~/.config/fish/completions/git-dom.fish
git-dom completions powershell > git-dom.ps1
```

## Configuration

All configuration is via `git config`:

| Key          | Default | Description                          |
|--------------|---------|--------------------------------------|
| `dom.root`   | `src`   | Root directory for `clone` paths     |
| `dom.commit` | `auto`  | Default commit mode for `pull`       |

## Colour

Colour output is on by default. Disable with:

- `--no-colour` flag
- `NO_COLOR=1` environment variable ([no-color.org](https://no-color.org/))

## License

MIT
