# Windows desktop preview

The desktop application reuses the TypeScript/Vite interface inside a Tauri
v2 WebView and uses the existing Rust core for calculations. The Windows
release executable compiles and opens its main window; the NSIS setup package
also builds. The visible AI Control Center is included in the production UI.
Installing/uninstalling the package and validating the new native UI/IPC
workflow through the packaged app remain outstanding.

## AI Control Center

The AI button sits immediately beside the theme control. It opens an
interactive workspace with a privacy-limited app-state view, current local
capability states, bounded session operation history, the existing
Rust-backed analyst, and the existing baseline/walk-forward scenario
calculation. Supported tools are restricted to read-only state inspection
and navigation; no model-generated commands or arbitrary file/network access
are accepted. Long-running WASM calculations are synchronous and cannot be
cancelled; the UI does not show fake progress.

Provider-backed quotes, history, company research and web search remain
unconfigured. The active desktop AI workspace uses the native Gemma 3 4B IT
Q4_K_M model through a loopback-only llama.cpp CPU process managed by Rust.
The Gemma weights and matching vision projector are downloaded on demand to a
local folder; the CPU runtime is bundled from the verified local
`models\gemma-3-4b\runtime\` directory. A release build therefore requires
that runtime directory to exist. The roughly 3.34 GB model assets are not
bundled in the installer. Their pinned source revision, exact sizes, GGUF
headers, and SHA-256 values are checked before loading. Loading is blocked
unless at least 6 GiB of free physical memory can be measured. Inference is
serialized with a 4096-token context; actual 16 GB target machine performance
is not verified.

The AI workspace provides local chat, explicit image input, user-authorized
single-frame screen capture with local Windows OCR, and a schema-validated
allow-list of local calculation tools. Repeated or background screen capture
is not performed.
Untrusted model output and image observations are not market evidence.
Calculation tools report their implementation method but are not
independently cross-verified and are never labelled confirmed financial
results. The application no longer includes a legacy external model-server
integration. Browser mode does not start the native inference runtime.

## Prerequisites

- Windows 10 or newer with the WebView2 runtime.
- Rust toolchain and the Windows MSVC target.
- Microsoft C++ Build Tools with the Windows SDK.
- Node.js/npm and `wasm-pack`.

See [INSTALLATION.md](INSTALLATION.md) for Rust/WASM and Node setup. Do not
install system toolchains without reviewing their size and system impact.

## Development

From `frontend\`:

```powershell
npm.cmd ci
npm.cmd run tauri:dev
```

Tauri starts Vite at `127.0.0.1:1420` for development only. The Vite host is
bound to loopback and uses a strict port.

## Release build

From `frontend\`:

```powershell
npm.cmd run tauri:build
```

This runs the production WASM build, TypeScript typecheck, Vite build, and
Tauri Windows NSIS bundling. The pinned CPU runtime is sourced from the local
model directory; model weights remain outside the installer. To compile only
the native executable without
building an installer:

```powershell
npm.cmd run tauri:build -- --no-bundle
```

The verified no-bundle build produced
`frontend\src-tauri\target\release\tredalix-ai-desktop.exe`. The verified
NSIS build produced
`frontend\src-tauri\target\release\bundle\nsis\TREDALIX AI_0.1.0_x64-setup.exe`.
The most recent verified Gemma-enabled NSIS installer is
`frontend\src-tauri\target-verify\release\bundle\nsis\TREDALIX AI_0.1.0_x64-setup.exe`
(18,249,007 bytes; 17.40 MiB). The rebuilt app window was observed during a
15-second launch check (27.9 MiB working set). Installer
installation/uninstallation, signed distribution, and end-to-end native
Gemma chat/vision remain unverified.

## Native Gemma manual verification

1. Launch the Windows desktop app and open **AI Control Center**.
2. Confirm the local model folder points to the directory containing the
   downloaded Gemma weights and projector; select **Verify assets**.
3. Select **Load local model** and wait for the UI to report the CPU model as
   loaded. If free RAM is below 6 GiB, close other applications and
   retry rather than terminating processes automatically.
4. If the memory guard permits loading, send a harmless Hinglish prompt such
   as `Namaste, ap offline assistant ho?` and confirm a local response and
   approximate token/s statistics appear.
5. Attach a non-sensitive test image and ask Gemma to describe only visible
   content. Treat its interpretation as fallible; the model output is not OCR
   or market evidence.
6. To test screen understanding, use **Capture and analyze one frame**, choose
   a non-sensitive window in the system chooser, and confirm the display stream
   closes as soon as the frame is captured. Use **Unload** when finished.

These packaged UI steps are guidance, not a claim that the complete Tauri
chat/vision IPC workflow or installer install/uninstall has been tested.
Session chat is not persisted.

## Local service and security model

The desktop app does not start a separate HTTP API server and does not bind a
loopback or LAN port. Its frontend calls a narrow Rust Tauri IPC command to
read host readiness, version, transport, and supported local capabilities.
The command returns diagnostics only; it does not expose credentials,
environment values, or file contents. The host lifecycle is the app process
lifecycle.

The release UI is bundled locally. Gemma inference uses a Rust-managed
loopback-only process; the frontend does not connect directly to it. Model
download is explicit and uses the pinned HTTPS model revision. No browser-side
network access or cloud AI provider is used for model chat. The single window
receives only the default Tauri permissions and the opener permission for
HTTP(S) URLs; it has no arbitrary filesystem or shell permission. Browser
mode continues to use the WebAssembly implementation without Tauri IPC.
External-link handling, WebView startup, and a complete desktop user workflow
still need native runtime smoke tests.

## Current limitations

- This is a local desktop shell, not a provider-backed market-data service.
- No login, telemetry, cloud sync, broker connectivity, or live provider
  credentials are added.
- CSV data is processed locally; the shell does not upload it.
- The system network indicator is only a browser network hint; it does not
  verify provider or internet-service connectivity.
- An executable compile is not a claim that the application has been
  launched, packaged, signed, or installed successfully.
- Android packaging and device support are out of scope and unimplemented.
- Native window/process inventory, application launching/switching, UI
  Automation, mouse/keyboard control, and a coding-agent workspace are not
  implemented.
