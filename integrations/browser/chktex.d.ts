/**
 * Browser/Node integration for ChkTeX WASM.
 *
 * Build WASM first: `../../tools/build-wasm.sh` (from repo root).
 */

export type DiagnosticKind = 'Message' | 'Warning' | 'Error';

export interface ChktexDiagnostic {
  number: number;
  kind: DiagnosticKind;
  file: string;
  line: number;
  column: number;
  length: number;
  message: string;
}

export interface ChktexResult {
  diagnostics: ChktexDiagnostic[];
  output: string;
  exitStatus: number;
  warnings: number;
  errors: number;
  toJSON: () => string;
}

export interface LintOptions {
  filename?: string;
  chktexrc?: string | null;
  verbosity?: number;
}

export interface InitOptions {
  pkgUrl?: string;
  wasmModule?: WebAssembly.Module | BufferSource;
}

export interface InitHandle {
  version: () => string;
  defaultChktexrc: () => string;
}

export interface EditorMarker {
  startLineNumber: number;
  startColumn: number;
  endLineNumber: number;
  endColumn: number;
  message: string;
  severity: number;
  code: string;
}

export function initChktex(options?: InitOptions): Promise<InitHandle>;
export function normalizeResult(result: unknown): ChktexResult;
export function lint(source: string, options?: LintOptions): Promise<ChktexResult>;
export function lintBytes(bytes: Uint8Array, options?: LintOptions): Promise<ChktexResult>;
export function toEditorMarkers(diagnostics: ChktexDiagnostic[]): EditorMarker[];

declare const _default: {
  initChktex: typeof initChktex;
  lint: typeof lint;
  lintBytes: typeof lintBytes;
  normalizeResult: typeof normalizeResult;
  toEditorMarkers: typeof toEditorMarkers;
};

export default _default;
