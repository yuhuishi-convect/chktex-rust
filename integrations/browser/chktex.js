/**
 * Browser/Node integration for ChkTeX WASM.
 *
 * Build WASM first: `../../tools/build-wasm.sh` (from repo root).
 */

/** @typedef {'Message' | 'Warning' | 'Error'} DiagnosticKind */

/**
 * @typedef {Object} ChktexDiagnostic
 * @property {number} number
 * @property {DiagnosticKind} kind
 * @property {string} file
 * @property {number} line
 * @property {number} column
 * @property {number} length
 * @property {string} message
 */

/**
 * @typedef {Object} ChktexResult
 * @property {ChktexDiagnostic[]} diagnostics
 * @property {string} output
 * @property {number} exitStatus
 * @property {number} warnings
 * @property {number} errors
 * @property {() => string} toJSON
 */

/**
 * @typedef {Object} LintOptions
 * @property {string} [filename='main.tex']
 * @property {string|null|undefined} [chktexrc]
 * @property {number} [verbosity=2]
 */

/** @type {import('../../pkg/chktex_wasm.js')} */
let wasm;
/** @type {Promise<void>|null} */
let initPromise = null;

/**
 * Resolve the wasm-bindgen glue module and initialize WASM when required.
 * @param {string} [pkgUrl]
 * @param {WebAssembly.Module|BufferSource} [wasmModule]
 */
async function loadWasmModule(pkgUrl, wasmModule) {
  const mod = pkgUrl
    ? await import(/* webpackIgnore: true */ pkgUrl)
    : await import('../../pkg/chktex_wasm.js');

  // Browser (`--target web`) exports init as default; Node (`--target nodejs`) sync-loads.
  if (typeof mod.default === 'function') {
    await mod.default(wasmModule);
    return mod;
  }

  return typeof mod.check === 'function' ? mod : mod.default;
}

/**
 * Initialize the WASM module. Safe to call multiple times.
 *
 * @param {{ pkgUrl?: string, wasmModule?: WebAssembly.Module|BufferSource }} [options]
 * @returns {Promise<{ version: () => string, defaultChktexrc: () => string }>}
 */
export async function initChktex(options = {}) {
  if (!initPromise) {
    initPromise = (async () => {
      wasm = await loadWasmModule(options.pkgUrl, options.wasmModule);
    })();
  }
  await initPromise;
  return {
    version: () => wasm.version(),
    defaultChktexrc: () => wasm.defaultChktexrc(),
  };
}

/**
 * Normalize a wasm-bindgen CheckResult into plain JS objects.
 * @param {import('../../pkg/chktex_wasm.js').CheckResult} result
 * @returns {ChktexResult}
 */
export function normalizeResult(result) {
  const diagnostics = result.diagnostics.map((d) => ({
    number: d.number,
    kind: /** @type {DiagnosticKind} */ (d.kind),
    file: d.file,
    line: Number(d.line),
    column: d.column,
    length: d.length,
    message: d.message,
  }));

  return {
    diagnostics,
    output: result.output,
    exitStatus: result.exitStatus,
    warnings: result.warnings,
    errors: result.errors,
    toJSON: () => result.toJSON(),
  };
}

/**
 * Lint LaTeX source text.
 *
 * @param {string} source
 * @param {LintOptions} [options]
 * @returns {Promise<ChktexResult>}
 */
export async function lint(source, options = {}) {
  await initChktex();
  const filename = options.filename ?? 'main.tex';
  const chktexrc = options.chktexrc ?? null;
  const verbosity = options.verbosity ?? 2;

  const raw =
    verbosity === 2 && (chktexrc === null || chktexrc === undefined)
      ? wasm.check(source, filename, undefined)
      : wasm.checkWithVerbosity(source, filename, chktexrc ?? undefined, verbosity);

  return normalizeResult(raw);
}

/**
 * Lint raw bytes (Uint8Array). Useful for binary-safe pipelines.
 *
 * @param {Uint8Array} bytes
 * @param {LintOptions} [options]
 * @returns {Promise<ChktexResult>}
 */
export async function lintBytes(bytes, options = {}) {
  await initChktex();
  const filename = options.filename ?? 'main.tex';
  const chktexrc = options.chktexrc ?? undefined;
  const raw = wasm.checkBytes(bytes, filename, chktexrc);
  return normalizeResult(raw);
}

/**
 * Map diagnostics to Monaco/editor-style markers.
 *
 * @param {ChktexDiagnostic[]} diagnostics
 * @returns {Array<{ startLineNumber: number, startColumn: number, endLineNumber: number, endColumn: number, message: string, severity: number, code: string }>}
 */
export function toEditorMarkers(diagnostics) {
  const severity = { Message: 1, Warning: 4, Error: 8 };
  return diagnostics.map((d) => ({
    startLineNumber: d.line,
    startColumn: d.column + 1,
    endLineNumber: d.line,
    endColumn: d.column + d.length + 1,
    message: `ChkTeX ${d.number}: ${d.message}`,
    severity: severity[d.kind] ?? 4,
    code: String(d.number),
  }));
}

export default {
  initChktex,
  lint,
  lintBytes,
  normalizeResult,
  toEditorMarkers,
};
