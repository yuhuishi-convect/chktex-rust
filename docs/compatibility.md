# Compatibility and TDD Baseline

The compatibility target is upstream ChkTeX from `/tmp/chktex-upstream`
when available. The audited upstream checkout was:

```text
b0c5352bd775136b7777356067f8726919cc9043
```

## Upstream Test Surface

Upstream has two main runtime fixtures:

- `Test.tex` plus `Test.*.out`: broad warning-catalog coverage across
  regex modes.
- `tests/run-tests.sh`: integration coverage for recursive `\input`,
  relative and absolute input paths, resource-file lookup, and `-S`
  command-line resource overrides.

`test-all.sh` runs `make check` under three regex configurations:

- no PCRE and no POSIX ERE
- POSIX ERE
- PCRE

The Rust port should keep those same dimensions visible. The default Rust
engine is byte regex, with optional `fancy-regex` for richer PCRE-like user
patterns.

## TDD Loop

Use oracle tests to capture upstream behavior before changing implementation:

```sh
CHKTEX_ORACLE=/tmp/chktex-upstream/chktex/chktex/chktex \
CHKTEX_UPSTREAM_DIR=/tmp/chktex-upstream/chktex/chktex \
cargo test -p chktex-cli --test oracle -- --ignored --nocapture
```

For narrower rule work, add a focused Rust unit test for the warning, then run
the matching oracle fixture command to verify exact line, column, message, and
format. The broad `fixture_output_matches_oracle_when_available` test is the
acceptance target, but it is too large for first diagnosis.

Current green gates:

- `cargo test -q`
- `CHKTEX_ORACLE=/tmp/chktex-upstream/chktex/chktex/chktex CHKTEX_UPSTREAM_DIR=/tmp/chktex-upstream/chktex/chktex cargo test -p chktex-cli --test oracle -- --ignored --nocapture`

The ignored oracle suite currently covers:

- upstream `Test.tex` exact stdout parity under `-mall -r -g0 -v5 -q`
- upstream recursive `tests/main.tex` inclusion fixture
- resource lookup order for XDG, HOME, LOGDIR, CHKTEXRC, and cwd `.chktexrc`
- `-S` command-line resource override visibility through `-d 4`
- warning/message/error exit status parity
- last-diagnostic exit status ordering for mixed warning/error runs, including
  multi-file order
- focused option parity for `-H`, `-x`, and `-V`
- `CmdLine`, `CmdSpaceStyle`, and `TeXInputs` resource behavior for focused
  compatibility cases
- missing `\input`/`\include` behavior, including warning 27 emission,
  suppression, continued checking, and legacy stderr warning output
- include scanning edge cases for unbraced EOF arguments and multiple include
  commands on one line
- `CmdLine` action behavior for `--version`, `--help`, and `--license`
- `CmdLine` accumulation across multiple rc files, including interaction with
  rc-sourced `-r`
- `CmdLine` runtime reset behavior where rc-sourced `-r` clears earlier argv
  options such as `-q` and `-v0`
- `CmdLine` option ordering where `-r` clears an earlier `-o` and permits a
  later output target
- non-quiet banner and summary output for clean, warning, and error runs
- exact action output for `--version`, `--help`, and `--license`
- focused `-d1` debug output for the warning/error/message table, including
  default muted statuses and CLI severity overrides
- focused `-d8` debug output for output format and boolean flags
- focused `-d16` runtime debug output for recursive `TeXInputs` file-search
  tracing
- focused `-d2` debug output for resource table summaries under reset/minimal
  resource configuration
- default-resource `-d2` debug output for resource table summaries
- focused `-d4` debug output for reset/minimal resource list dumps
- default-resource `-d4` debug output for resource list dumps
- combined debug bitfields for `-d6` and `-d31`, including default-empty
  list entries and runtime trace ordering
- failed recursive `TeXInputs` traversal under `-d16`, including the legacy
  directory-open warning
- custom case-insensitive `-d2`/`-d4` debug output for focused resources,
  including `Abbrev` / `AbbrevCase` first-letter expansion
- EOF warning controls for warnings 15, 16, and 17, including suppression and
  severity options in focused fixtures
- EOF command-delimited math cases for `\[` and `\(`
- EOF ConTeXt warning 48 cases, including source line reporting, suppression,
  severity display, non-fatal exit status, and interaction with open math mode
- `\left`/`\right` delimiter spacing around W36/W37 in focused cases
- output-file behavior for `-o`, including default backup creation, backup
  note formatting, redirected diagnostics, preserved `.bak` contents, and
  explicit `-b0` backup disabling
- output-file edge cases for pre-existing backup overwrite and directory
  output-open failure
- output-file permission-denied open failure under a read-only output
  directory
- output-file backup rename failure when the `.bak` target cannot be removed

## Recent Compatibility Fixes

- Routed `-H` into checker configuration and suppressed header diagnostics
  before `\begin{document}` when disabled.
- Routed `-x` into verb wiping so `-x0` checks text inside `\verb`.
- Routed `-V` pipe verbosity through `stdout` terminal detection.
- Added EOF-origin W15 reporting for an unmatched `\begin{document}` while
  keeping include-segment output aligned with upstream.
- Preserved EOF-origin diagnostics separately from display line numbers so
  output ordering can match upstream.
- Applied `CmdLine` resource options after argv so rc defaults match upstream
  precedence in focused cases.
- Matched rc-sourced `CmdLine` action semantics where `--version` and
  `--license` are terminal actions, while `--help` prints help and continues
  checking.
- Matched repeated `CmdLine` resource merging across rc files so command-line
  fragments accumulate in upstream order.
- Matched rc-sourced `CmdLine` reset behavior so `-r` clears runtime options
  parsed earlier from argv while preserving resource data and warning changes.
- Matched upstream parser reset semantics for output options so `-o first -r
  -o second` is accepted and writes only to the later output target.
- Implemented `CmdSpaceStyle` handling for W12/W13 command-adjacent spacing.
- Added `TeXInputs` include lookup for direct path entries and recursive `//`
  path entries.
- Matched missing include behavior so failed input opens emit warning 27 when
  enabled, print the legacy stderr warning, and continue checking the parent
  file.
- Matched upstream exit status ordering where the last non-message diagnostic
  determines the process status instead of the maximum severity seen.
- Matched include target scanning for unbraced EOF arguments and same-line
  include processing order.
- Added legacy banner/version text and non-quiet warning/error summary output.
- Replaced placeholder help/license output with upstream-compatible action
  text.
- Added focused debug warning-table output for `FLG_DbgWarnInfo` / `-d1`,
  including legacy raw-byte text and user/system-muted status reporting.
- Added focused debug flag output for `FLG_DbgOtherInfo` / `-d8`.
- Added focused runtime debug output for `FLG_DbgRunTime` / `-d16`, matching
  upstream recursive `TeXInputs` search tracing.
- Added focused `FLG_DbgHashInfo` / `-d2` resource-summary output for reset
  resource runs.
- Matched default-resource `-d2` summary counts for star-expanded environment
  lists and upstream hash-usage display for large default lists.
- Added focused `FLG_DbgHashContent` / `-d4` list-dump output for reset
  resource runs.
- Matched combined debug resource output so `-d2` and `-d4` share one
  upstream-ordered table pass instead of duplicated passes.
- Matched debug output for default-empty lists such as `CmdLine` and
  `TeXInputs` when rc entries are appended after the leading empty item.
- Matched failed recursive `TeXInputs` traversal reporting so unreadable
  search directories emit upstream's runtime trace and directory-open warning.
- Matched default-resource `-d4` list dump formatting, including display-only
  stripping of argument specs for `WipeArg` and `NoCharNext`, and upstream
  ordering for star-expanded environment entries.
- Matched custom case-insensitive debug list handling for focused resources,
  including upstream's first-letter expansion of `AbbrevCase` entries into
  `Abbrev`.
- Expanded EOF warning parity for W15-W17: open LaTeX environments, unmatched
  delimiters, math-mode EOF, ordering, displayed line numbers, and non-fatal
  exit status.
- Matched W16 command-delimited math EOF behavior for `\[` and `\(`, including
  upstream's empty filename rendering in that path.
- Matched W48 ConTeXt EOF behavior so unmatched `\start...` reports the
  original source line, respects suppression, sorts before W16 at EOF, and
  remains non-fatal even when displayed as an error.
- Suppressed false W36 before delimiters immediately following `\left` /
  `\right`.
- Aligned `-o` output-file handling with upstream default backup behavior and
  legacy backup-renaming notes.
- Matched output-file overwrite and open-failure behavior, including not
  backing up directories and using the legacy output-open error.
- Added output-file permission-denied oracle coverage for read-only output
  directories.
- Matched backup rename failure behavior and legacy error text when the
  `.bak` target cannot be removed before backup creation.

## Design Alignment

Upstream is a byte-oriented scanner, not a full TeX parser. Its behavior comes
from a line normalization pass, a shallow state machine, resource-driven word
lists, and warning contexts. The Rust implementation should keep that model:

- preserve byte offsets and line-oriented processing
- keep scanner state explicit rather than global
- use parsed resource lists to drive command/environment behavior
- avoid a full TeX AST unless a specific compatibility gap requires it

The active implementation should converge on one scanner path. Keeping multiple
partially overlapping checkers makes parity hard to reason about; compatibility
tests should decide which path owns each warning before refactoring.

## Remaining Known Gaps

- Debug output coverage includes warning-table, resource-summary,
  resource-list, flag dumps, recursive-search runtime tracing, and combined
  bitfields through `-d31`, including failed recursive directory traversal.
  Remaining risk is in unusual platform-specific filesystem behavior.
- `CmdLine` coverage includes option precedence, common formatting / warning
  switches, action options, multi-rc accumulation, and focused `-r` runtime
  reset behavior, including reset-split duplicate output options. Remaining
  risk is in recursive `CmdLine` additions from rc files loaded by `CmdLine`
  itself, which upstream appears not to re-apply in the focused probes.
- `TeXInputs` coverage includes direct and recursive search entries plus
  missing include failures, focused include-scanning edge cases, and upstream
  `.tex` fallback for names that already contain another extension; exact
  kpsewhich integration and platform-specific path syntax remain uncovered.
  Focused include-scanning coverage now also includes nested missing includes
  where upstream reports W27 from the included file rather than resolving
  relative to that included file's directory.
- EOF warning coverage now includes command-delimited math, ConTeXt EOF W48,
  and selected mixed parser-state combinations, but is still not exhaustive for
  every parser-state interaction.
- Output-file coverage now checks default backup, explicit backup disabling,
  pre-existing backup overwrite, directory output-open failure, and
  permission-denied output directories, plus backup rename failures caused by
  an unremovable `.bak` target. Remaining filesystem risk is in platform-
  specific edge cases such as symlink behavior.
