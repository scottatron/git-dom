# Releasing git-dom

Releases are managed with [cargo-release](https://github.com/crates-io/cargo-release) and built automatically by GitHub Actions.

## Prerequisites

```sh
cargo install cargo-release
```

## Creating a Release

1. **Make sure `main` is clean and up to date:**

   ```sh
   git checkout main
   git pull
   ```

2. **Run cargo-release** with the desired bump level:

   ```sh
   # Patch release (0.1.0 → 0.1.1)
   cargo release patch --execute

   # Minor release (0.1.1 → 0.2.0)
   cargo release minor --execute

   # Major release (0.2.0 → 1.0.0)
   cargo release major --execute
   ```

   This will:
   - Bump the version in `Cargo.toml`
   - Update `Cargo.lock`
   - Commit with message `Release <version>`
   - Create a `v<version>` tag
   - Push the commit and tag to GitHub

3. **GitHub Actions takes over:**

   The [release workflow](.github/workflows/release.yml) triggers on the `v*` tag and:
   - Builds release binaries for macOS (aarch64, x86_64) and Linux (aarch64, x86_64)
   - Creates a GitHub Release with auto-generated release notes
   - Attaches the binary tarballs to the release

4. **Verify** at [github.com/scottatron/git-dom/releases](https://github.com/scottatron/git-dom/releases).

## Dry Run

To preview what cargo-release will do without making changes:

```sh
cargo release patch
```

(Omit `--execute` for a dry run.)

## Notes

- `publish = false` in `Cargo.toml` — we don't publish to crates.io yet
- Tags are unsigned (`sign-tag = false`) — enable with `sign-tag = true` if you set up GPG
