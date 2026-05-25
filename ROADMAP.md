# ChkTeX Rust Rewrite Roadmap

## High-Level Goal

Rewrite ChkTeX in Rust as a self-contained, cross-platform command-line tool
while preserving the behavior users depend on from upstream ChkTeX.

The primary compatibility target is the canonical upstream C implementation:

- Repository: https://git.savannah.nongnu.org/git/chktex.git
- Web: https://cgit.git.savannah.gnu.org/cgit/chktex.git/
- Homepage: https://www.nongnu.org/chktex/

The rewrite should prioritize:

- CLI compatibility with `chktex`.
- `.chktexrc` compatibility.
- Stable warning numbers, severities, messages, output formats, and exit codes.
- Byte-oriented processing, matching upstream's current behavior.
- Self-contained distributable binaries for Linux, Windows, and macOS.
- Minimal external/native dependencies.
- Tests-first development: establish correctness and C-implementation
  compatibility tests before porting behavior, then use TDD for each rewrite
  increment.

The Rust version should not initially try to become a full TeX parser. The
right model is a byte lexer plus shallow TeX-aware state machine, with
compatibility tests against upstream.

## Core Design Decisions

- Use byte-oriented input internally. Do not require UTF-8 for normal checking.
- Use `regex::bytes` as the default regex engine.
- Add optional `fancy-regex` support for richer UTF-8-only user regexes.
- Keep PCRE/kpathsea/native integrations optional, if they are added later.
- Model scanner state explicitly instead of using global mutable state.
- Keep internal rules token/state based where practical; reserve regex for
  user-facing config features like `UserWarnRegex` and `Silent [...]`.

## Implementation Plan

### 1. Test Baseline First

- Build or provide an upstream C `chktex` oracle binary.
- Import upstream fixtures and expected outputs before implementing Rust
  behavior.
- Create the compatibility test harness early, even if the Rust binary only
  returns placeholder output at first.
- Define the expected stdout, stderr, and exit-code comparison rules.
- Use this baseline to drive all subsequent work.

The rewrite should proceed test-first:

1. Add or import a failing compatibility/correctness test.
2. Implement the smallest Rust behavior needed to pass it.
3. Compare against the upstream C implementation where possible.
4. Refactor only after the compatibility test passes.

### 2. Repository Scaffold

- Create a Cargo workspace.
- Add a CLI crate for the `chktex` executable.
- Add a core crate for scanner, config, diagnostics, and rules.
- Add a small regex abstraction module.
- Add license and project metadata.

Suggested structure:

```text
crates/
  chktex-core/
  chktex-cli/
tests/
  fixtures/
```

### 3. Compatibility Baseline

- Vendor or script-fetch upstream test fixtures.
- Add an oracle test harness that can compare Rust output against an upstream C
  `chktex` binary when available.
- Record the upstream commit used for compatibility testing.

Initial fixture set:

- `Test.tex`
- `Test.*.out`
- `tests/main.tex`
- `tests/main.expected`
- generated rc lookup fixtures from upstream `tests/run-tests.sh`

### 4. CLI Compatibility

- Implement legacy argument parsing before using higher-level CLI conveniences.
- Preserve compact forms such as `-mall`, `-v0`, `-g0`, `-I1`.
- Preserve long options and optional boolean argument behavior.
- Match stdout/stderr behavior, banner behavior, and exit codes.

### 5. Resource File Parser

- Implement `.chktexrc` parsing:
  - keywords
  - `{}` lists
  - `[]` case-insensitive lists
  - `=`
  - quoted strings
  - `!` escapes
  - `#` comments
- Preserve default resource values and list semantics.
- Implement command-line resource overrides via `-S`.

### 6. File and Config Discovery

- Implement config search order:
  - system rc
  - XDG config
  - `HOME`
  - `LOGDIR`
  - `CHKTEXRC`
  - current directory
- Implement TeX input search paths for `\input`.
- Keep kpathsea integration optional for future TeX Live builds.

### 7. Lexer and Shallow Parser

- Implement a byte lexer for TeX-like input.
- Track spans as byte offsets to preserve upstream column behavior.
- Model tokens such as:
  - commands
  - groups
  - optional arguments
  - math shifts
  - comments
  - spaces
  - punctuation
  - text bytes
- Add lexer modes for verbatim/wiped regions.

### 8. Rule Engine

- Port warning behavior in focused groups.
- Preserve warning numbers and messages.
- Keep rule state explicit and testable.
- Port suppression behavior:
  - per-line suppressions
  - per-file suppressions
  - user warning suppressions

### 9. Regex Support

- Define an internal regex engine abstraction.
- Default engine: `regex::bytes`.
- Optional feature: `fancy-regex`.
- Default behavior should skip `PCRE:` patterns unless a rich/compatible engine
  is enabled.
- Preserve ChkTeX matching semantics:
  - run matches against the current byte slice from `offset`
  - add `offset` back to reported spans
  - advance by full match end
  - stop on empty matches

### 10. Output Formatting

- Implement ChkTeX format placeholders.
- Preserve default `OutFormat` entries.
- Test `-v0`, `-v1`, `-v2`, `-v3`, `-v4`, `-v5`, `-v6`, `-f`, and `-V`.
- Preserve byte-offset based underline behavior.

### 11. Packaging

- Produce standalone release archives for:
  - Linux x86_64 musl
  - Linux aarch64 musl
  - macOS x86_64
  - macOS aarch64
  - Windows x86_64 MSVC
- Include:
  - `chktex` or `chktex.exe`
  - default `chktexrc`
  - license
  - README
  - manpage or generated command help

## Verification Plan

Verification is part of the implementation loop, not a final cleanup step. New
behavior should normally start as a failing test, preferably a differential test
against the C implementation when upstream behavior is known.

### Unit Tests

- Resource parser tokenization and parsing.
- Escape handling in rc files.
- CLI argument parsing edge cases.
- Regex prefix handling:
  - unprefixed patterns
  - `PCRE:` patterns
  - `POSIX:` patterns
  - initial `(?#message)` comments
- Lexer token spans and byte offsets.
- Individual rule behavior.

### Integration Tests

- Run Rust `chktex` against upstream fixtures.
- Compare stdout, stderr, and exit code for each supported mode.
- Cover stdin input and file input.
- Cover nested `\input` behavior.
- Cover config lookup through environment variables.
- Cover output redirection and backup behavior.

### Oracle/Differential Tests

When an upstream C binary is available:

```text
chktex-oracle [args] input.tex > c.out 2> c.err
target/debug/chktex [args] input.tex > rust.out 2> rust.err
diff -u c.out rust.out
diff -u c.err rust.err
compare exit codes
```

Run this across:

- default fixtures
- custom rc files
- warning enable/disable combinations
- output format variants
- malformed input
- long lines
- non-UTF-8 byte input

### Cross-Platform Tests

- Run normal tests on Linux, Windows, and macOS.
- Add focused platform tests for:
  - path separators
  - config lookup
  - line endings
  - stdin/stdout behavior
  - executable packaging

### Fuzz and Robustness Tests

- Fuzz rc parsing.
- Fuzz lexer input.
- Fuzz shallow parser state transitions.
- Ensure malformed input reports errors or warnings without panics.

### Release Verification

- Build release binaries for all target platforms.
- Verify each archive runs without native library installation.
- Run smoke tests from each archive.
- Confirm `--version`, `--help`, stdin mode, file mode, and rc loading.

## First Implementation Steps

1. Add upstream fixture import and oracle test harness.
2. Create the first failing compatibility test against the C implementation.
3. Create the Cargo workspace and crates.
4. Add the regex abstraction with `regex::bytes` default.
5. Add optional `fancy-regex` behind a feature flag.
6. Add tests for ChkTeX-style regex offset behavior.
7. Implement `.chktexrc` parser using TDD.
8. Implement CLI compatibility parser using TDD.
