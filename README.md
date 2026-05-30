# ChkTeX (Rust)

[![CI](https://github.com/yuhuishi-convect/chktex-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/yuhuishi-convect/chktex-rust/actions/workflows/ci.yml)

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

## CI and releases

GitHub Actions runs on every push and pull request:

- `fmt` · `clippy` · workspace tests
- Release builds on Linux, macOS, and Windows
- Upstream oracle compatibility suite (Linux)

Create a release by pushing a version tag:

```sh
git tag v0.1.0
git push origin v0.1.0
```

That builds archives for Linux (x86_64 and aarch64), Windows (x86_64), and macOS (Apple Silicon), then publishes them to GitHub Releases. Each archive contains the binary, default `chktexrc`, `LICENSE`, and `README.md`.

Manual release (workflow dispatch): run the **Release** workflow from the Actions tab and provide a tag such as `v0.1.0` (the tag must already exist on the branch).

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
  chktex-wasm/   Browser/WASM bindings (wasm-bindgen)
tests/fixtures/  Upstream-derived fixtures (chktexrc, inclusion tests)
examples/wasm/   Browser demo page
tools/           Oracle setup, cross-build scripts, diff helpers
docs/            Compatibility notes and TDD baseline
```

## WebAssembly / browser

`chktex-core` builds for `wasm32-unknown-unknown`. The `chktex-wasm` crate exposes
typed JavaScript bindings (via wasm-bindgen) for linting a LaTeX buffer in the browser
(no filesystem or `\input` resolution in this mode).

Build WASM packages:

```sh
cargo install wasm-pack   # once
make build-wasm           # writes pkg/ (browser) and pkg-node/ (Node tests)
make test-wasm            # build + run JS integration smoke tests
```

Try the demo:

```sh
python -m http.server 8000
# open http://localhost:8000/examples/wasm/index.html
```

### JS integration layer

`integrations/browser/chktex.js` wraps the raw wasm-bindgen glue with a small API:

```javascript
import { initChktex, lint, lintBytes, toEditorMarkers } from "./integrations/browser/chktex.js";

await initChktex();
const result = await lint("\\foo x", { filename: "main.tex" });
// result.diagnostics, result.output, result.exitStatus, result.warnings, result.errors

const markers = toEditorMarkers(result.diagnostics); // Monaco / VS Code shape
```

Options for `lint` / `lintBytes`:

- `filename` — virtual path shown in diagnostics (default `main.tex`)
- `chktexrc` — custom `.chktexrc` text, or omit/`null` for embedded defaults
- `verbosity` — ChkTeX `-vN` output format index (default `2`)

For bundlers or Node tests, point at a specific glue build:

```javascript
await initChktex({ pkgUrl: new URL("../../pkg-node/chktex_wasm.js", import.meta.url).href });
```

TypeScript types: `integrations/browser/chktex.d.ts`.

GitHub Releases include a `chktex-wasm-{version}.tar.gz` browser bundle (`pkg/`, JS wrapper, demo). Extract, serve over HTTP, and open `example.html` — see `WASM.md` inside the archive.

### Low-level wasm-bindgen API

After `await init()` from `pkg/chktex_wasm.js`:

```javascript
import init, { check, checkBytes, version, defaultChktexrc } from "./pkg/chktex_wasm.js";

await init();
const result = check("\\foo x", "main.tex", undefined);
// result.diagnostics (CheckDiagnostic[]), result.output, result.exitStatus
result.toJSON(); // optional JSON string
```

Custom `.chktexrc` text can be passed as the third argument instead of `undefined`.

## Library usage

```rust
use chktex_core::session::{check_buffer, default_resources, CheckOptions};

let resources = default_resources();
let result = check_buffer("doc.tex", br"\documentclass{article}", &resources, &CheckOptions::default());
// result.diagnostics, result.formatted, result.summary
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
| `package-release` | Create release archive (see `tools/package-release.sh`) |
| `build-wasm` | Build browser and Node WASM packages (`pkg/`, `pkg-node/`) |
| `test-wasm` | Build WASM and run JS integration smoke tests |
| `package-wasm-release` | Create browser WASM release tarball (`NAME=chktex-wasm-0.1.1`) |
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
