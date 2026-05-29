# ChkTeX (Rust)

A Rust rewrite of [ChkTeX](https://www.nongnu.org/chktex/), the LaTeX semantic checker. The goal is a self-contained, cross-platform `chktex` binary that preserves upstream CLI behavior, warning numbers, `.chktexrc` format, output formatting, and exit codes.

This project tracks upstream ChkTeX 1.7.x compatibility using differential tests against the legacy C implementation.

## Features

- All 49 upstream warnings implemented in a byte-oriented scanner
- Legacy CLI flags (`-mall`, `-v0`–`-v6`, `-d1`–`-d31`, `-S`, `-o`, …)
- `.chktexrc` parsing and config discovery (XDG, `HOME`, `LOGDIR`, `CHKTEXRC`, cwd)
- `\input` / `\include` with `TeXInputs` search
- Library crate (`chktex-core`) plus `chktex` CLI binary

## Requirements

- Rust stable (see `rust-toolchain.toml`)
- For oracle tests: `git`, `gcc`, `make`, `perl`, autotools
- For Windows cross-compile from Linux: [`cargo-xwin`](https://github.com/rust-cross/cargo-xwin)

## Quick start

```sh
# Build
cargo build --release -p chktex-cli

# Run
./target/release/chktex -mall -v5 mydoc.tex

# Or via Makefile
make release
make run FILE=mydoc.tex ARGS="-mall -v5"
```

Stdin is supported when no file arguments are given:

```sh
echo '\documentclass{article}' | ./target/release/chktex -mall -v0
```

## Testing

### Unit and integration tests

```sh
make test
# or
cargo test --workspace
```

### Oracle tests (compatibility with upstream C)

Oracle tests compare Rust stdout, stderr, and exit codes against a built upstream `chktex` binary.

```sh
# One-time setup: clone upstream, build C oracle, write target/oracle.env
make oracle-setup

# Run the full differential suite (35 tests)
make oracle-tests

# Setup + test in one step
make oracle-setup-tests
```

Compare warning output line-by-line on upstream `Test.tex`:

```sh
make diff-warnings
```

See [docs/compatibility.md](docs/compatibility.md) for the TDD baseline, audited upstream commit, and known gaps.

## Cross-compilation (Windows)

From Linux, build a Windows `.exe` with the MSVC target (recommended):

```sh
cargo install cargo-xwin --locked
make release-windows
```

Output: `target/x86_64-pc-windows-msvc/release/chktex.exe`

Package the binary with the default resource file:

```sh
make package-windows
# -> target/windows-msvc/chktex.exe + chktexrc
```

For the GNU/mingw ABI instead:

```sh
make release-windows WINDOWS_FLAVOR=gnu
```

Requires `mingw-w64-gcc` (e.g. `sudo pacman -S mingw-w64-gcc` on Arch).

## Project layout

```text
crates/
  chktex-core/   Library: lexer, checker, .chktexrc parser, diagnostics
  chktex-cli/    `chktex` executable and oracle integration tests
tests/fixtures/  Upstream-derived fixtures (chktexrc, inclusion tests)
tools/           Oracle setup, cross-build scripts, diff helpers
docs/            Compatibility notes and TDD baseline
```

## Library usage

```rust
use chktex_core::{
    checker::{check_document, CheckerConfig},
    resource::{parse_resource, ResourceSet},
};

let resources = ResourceSet::default();
let config = CheckerConfig::from_resources(&resources);
let diagnostics = check_document("doc.tex", br"\documentclass{article}", &config);
```

## Makefile targets

Run `make` or `make help` for the full list. Common targets:

| Target | Description |
|--------|-------------|
| `build` / `release` | Build debug or release binary |
| `test` | Run workspace tests |
| `oracle-setup` | Build upstream C oracle |
| `oracle-tests` | Run differential oracle suite |
| `release-windows` | Cross-compile `chktex.exe` |
| `package-windows` | Stage exe + default `chktexrc` |
| `clippy` / `fmt` | Lint and format |

## Optional features

`chktex-core` supports optional regex backends via Cargo features:

- `regex-bytes` (default) — `regex::bytes`, matches upstream byte-oriented patterns
- `fancy-regex` — richer UTF-8 user regexes; enables `PCRE:` patterns in `.chktexrc`

```sh
cargo build -p chktex-cli --features chktex-core/fancy-regex
```

## Upstream reference

- Upstream repository: https://git.savannah.nongnu.org/git/chktex.git
- Compatibility baseline commit: `b0c5352bd775136b7777356067f8726919cc9043`
- Rewrite plan: [ROADMAP.md](ROADMAP.md)

## License

This project is licensed under **GPL-2.0-or-later**, the same terms as upstream ChkTeX. See [LICENSE](LICENSE) for the full text.

Upstream ChkTeX is Copyright (C) 1995-96 Jens T. Berger Thielemann.
