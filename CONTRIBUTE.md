# Contribute

This file is for development work on atom. User-facing usage is documented in
[README.md](README.md).

## Development Environment

Build from source:

```bash
cargo build --release
```

The repository also supports Dev Containers. Open the repository in VS Code and
choose `Reopen in Container` to use a container with Rust, make, just, Poppler,
Tesseract, and LibreOffice.

## Common Commands

Run the Rust test suite:

```bash
make test
```

Run Markdown linting:

```bash
make lint
```

Run Rust static checks:

```bash
make clippy
```

Run fixture regression checks:

```bash
make regression-test
```

Run the full local verification target:

```bash
make verify
```

Run real-document evaluation and performance checks separately from the normal
test loop:

```bash
make bench
```

`just` provides shorter entry points for common workflows:

```bash
just test
just eval
```

## Release Checks

Before preparing a release, run the relevant verification commands for the
change set. For conversion behavior changes, include `make regression-test`.
For distribution or workflow changes, include the distribution test:

```bash
cargo test --test distribution
```

Release ZIP archives must include both the executable and
`config.toml.example` so users can create `~/.atom/config.toml` without
checking out the repository.

The release workflow also publishes `atom-<version>-source.tar.gz` and its
`.sha256` file. The `update-homebrew-tap` release job uses that source archive
to update `Formula/atom.rb` in `izuno4t/homebrew-tap`, so Homebrew can build
the package with its Rust build dependency instead of depending on a
CPU-specific binary ZIP.

Set either the `HOMEBREW_TAP_GITHUB_TOKEN` or `TAP_GITHUB_TOKEN` repository
secret before publishing a tag. The token must be able to push to
`izuno4t/homebrew-tap`.

Package managers should install the example config through the `install` target:

```bash
make install INSTALL_DIR=/path/to/bin SHARE_DIR=/path/to/share/atom
```

## Homebrew Tap Release

The Homebrew tap repository is
`https://github.com/izuno4t/homebrew-tap`. The formula should use the source
archive published by the atom release workflow:

```text
https://github.com/izuno4t/atom/releases/download/v0.1.1/atom-0.1.1-source.tar.gz
```

The release workflow updates `Formula/atom.rb` automatically. After the workflow
finishes, verify the tap:

```bash
brew audit --strict --online izuno4t/tap/atom
brew install --build-from-source izuno4t/tap/atom
brew test izuno4t/tap/atom
```
