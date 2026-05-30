//! WASM bindings for running ChkTeX in browsers and other WASM hosts.
//!
//! Build with `tools/build-wasm.sh` or:
//!   wasm-pack build crates/chktex-wasm --target web --out-dir ../../pkg

use chktex_core::{
    diagnostic::DiagnosticKind,
    resource::{ResourceSet, parse_resource},
    session::{CheckOptions, CheckOutput, check_buffer, default_resources},
};
use wasm_bindgen::prelude::*;

/// One ChkTeX diagnostic exposed to JavaScript.
#[wasm_bindgen]
#[derive(Clone)]
pub struct CheckDiagnostic {
    number: i32,
    kind: String,
    file: String,
    line: i64,
    column: usize,
    length: usize,
    message: String,
}

#[wasm_bindgen]
impl CheckDiagnostic {
    #[wasm_bindgen(getter)]
    pub fn number(&self) -> i32 {
        self.number
    }

    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        self.kind.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn file(&self) -> String {
        self.file.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn line(&self) -> i64 {
        self.line
    }

    #[wasm_bindgen(getter)]
    pub fn column(&self) -> usize {
        self.column
    }

    #[wasm_bindgen(getter, js_name = length)]
    pub fn len(&self) -> usize {
        self.length
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

/// Lint result exposed to JavaScript.
#[wasm_bindgen]
pub struct CheckResult {
    diagnostics: Vec<CheckDiagnostic>,
    output: String,
    exit_status: u8,
    warnings: usize,
    errors: usize,
}

#[wasm_bindgen]
impl CheckResult {
    #[wasm_bindgen(getter)]
    pub fn diagnostics(&self) -> Vec<CheckDiagnostic> {
        self.diagnostics.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn output(&self) -> String {
        self.output.clone()
    }

    #[wasm_bindgen(getter, js_name = exitStatus)]
    pub fn exit_status(&self) -> u8 {
        self.exit_status
    }

    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> usize {
        self.warnings
    }

    #[wasm_bindgen(getter)]
    pub fn errors(&self) -> usize {
        self.errors
    }

    /// Serialize the full result as JSON (for logging or storage).
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "diagnostics": self.diagnostics.iter().map(|d| serde_json::json!({
                "number": d.number,
                "kind": d.kind,
                "file": d.file,
                "line": d.line,
                "column": d.column,
                "length": d.length,
                "message": d.message,
            })).collect::<Vec<_>>(),
            "output": self.output,
            "exit_status": self.exit_status,
            "warnings": self.warnings,
            "errors": self.errors,
        })
        .to_string()
    }
}

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Return the ChkTeX crate version string.
#[wasm_bindgen]
pub fn version() -> String {
    chktex_core::PACKAGE_VERSION.to_string()
}

/// Return the embedded default `.chktexrc` text.
#[wasm_bindgen(js_name = defaultChktexrc)]
pub fn default_chktexrc() -> String {
    include_str!("../../../tests/fixtures/upstream/chktexrc").to_string()
}

/// Lint UTF-8 LaTeX source. Pass `undefined`/`null` for `chktexrc` to use defaults.
#[wasm_bindgen]
pub fn check(
    source: &str,
    filename: &str,
    chktexrc: Option<String>,
) -> Result<CheckResult, JsValue> {
    run_check(
        source.as_bytes(),
        filename,
        chktexrc,
        CheckOptions::default(),
    )
}

/// Lint raw document bytes (preserves non-UTF-8 bytes).
#[wasm_bindgen(js_name = checkBytes)]
pub fn check_bytes(
    source: &[u8],
    filename: &str,
    chktexrc: Option<String>,
) -> Result<CheckResult, JsValue> {
    run_check(source, filename, chktexrc, CheckOptions::default())
}

/// Lint with an explicit `-vN` output format index (`OutFormat` list entry).
#[wasm_bindgen(js_name = checkWithVerbosity)]
pub fn check_with_verbosity(
    source: &str,
    filename: &str,
    chktexrc: Option<String>,
    verbosity: i64,
) -> Result<CheckResult, JsValue> {
    run_check(
        source.as_bytes(),
        filename,
        chktexrc,
        CheckOptions {
            verbosity,
            ..CheckOptions::default()
        },
    )
}

fn run_check(
    input: &[u8],
    filename: &str,
    chktexrc: Option<String>,
    options: CheckOptions,
) -> Result<CheckResult, JsValue> {
    let resources = load_resources(chktexrc.as_deref())?;
    let result = check_buffer(filename, input, &resources, &options);
    Ok(into_check_result(result))
}

fn load_resources(chktexrc: Option<&str>) -> Result<ResourceSet, JsValue> {
    match chktexrc {
        Some(text) => parse_resource(text).map_err(|err| JsValue::from_str(&err.to_string())),
        None => Ok(default_resources()),
    }
}

fn into_check_result(result: CheckOutput) -> CheckResult {
    CheckResult {
        diagnostics: result
            .diagnostics
            .iter()
            .map(|diag| CheckDiagnostic {
                number: diag.number,
                kind: diagnostic_kind_name(diag.kind).to_string(),
                file: diag.file.clone(),
                line: diag.line,
                column: diag.column,
                length: diag.len,
                message: diag.message.clone(),
            })
            .collect(),
        output: String::from_utf8_lossy(&result.formatted).into_owned(),
        exit_status: result.summary.exit_status,
        warnings: result.summary.warnings,
        errors: result.summary.errors,
    }
}

fn diagnostic_kind_name(kind: DiagnosticKind) -> &'static str {
    kind.as_str()
}

mod console_error_panic_hook {
    use std::panic::{self, PanicHookInfo};
    use std::sync::Once;
    use wasm_bindgen::prelude::*;

    static SET_HOOK: Once = Once::new();

    pub fn set_once() {
        SET_HOOK.call_once(|| {
            panic::set_hook(Box::new(console_error));
        });
    }

    fn console_error(info: &PanicHookInfo<'_>) {
        let message = info.to_string();
        console_error_1(&JsValue::from_str(&message));
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console, js_name = error)]
        fn console_error_1(msg: &JsValue);
    }
}
