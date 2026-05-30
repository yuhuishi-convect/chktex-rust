# ChkTeX WASM browser bundle

This archive contains prebuilt browser artifacts — no Rust or wasm-pack required.

## Quick start

```sh
tar xzf chktex-wasm-*.tar.gz
cd chktex-wasm-*
python -m http.server 8000
# open http://localhost:8000/example.html
```

Do not open `example.html` via `file://`; browsers block ES module and WASM loading from disk.

## Layout

```
pkg/                      wasm-bindgen output (chktex_wasm.js + .wasm)
integrations/browser/     JS wrapper (chktex.js, chktex.d.ts)
example.html              interactive demo
```

## Use in your app

```javascript
import { initChktex, lint, toEditorMarkers } from "./integrations/browser/chktex.js";

await initChktex();
const result = await lint(latexSource, { filename: "main.tex" });
```

If your app layout differs, pass an explicit glue URL:

```javascript
await initChktex({ pkgUrl: new URL("./pkg/chktex_wasm.js", import.meta.url).href });
```

Low-level wasm-bindgen API (without the wrapper):

```javascript
import init, { check } from "./pkg/chktex_wasm.js";

await init();
const result = check(source, "main.tex", undefined);
```

## Hosting notes

- Serve `.wasm` with MIME type `application/wasm`.
- If JS and WASM are on different origins, enable CORS on both.
- Single-buffer mode only: no `\input`/`\include` or filesystem config discovery.

Custom `.chktexrc` text can be passed via `lint(source, { chktexrc: "..." })`.
