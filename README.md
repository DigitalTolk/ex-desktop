# ex Desktop

Native desktop client for [ex](https://github.com/DigitalTolk/ex) — a self-hosted team chat application.

Built with **Tauri v2** (Rust) and the **ex React frontend**. Runs on macOS, Windows, and Linux from a single codebase.

## Features

- Native OS notifications with channel mute support
- System tray with unread badge
- Tokens stored in the OS keychain (not localStorage)
- OIDC / SSO login via system browser with deep-link callback
- Auto-updater (silent background updates)
- Native file picker and drag-and-drop uploads
- Window state persistence (remembers size and position)
- Multi-server support (connect to any self-hosted ex instance)

## Prerequisites

You need a running [ex server](https://github.com/DigitalTolk/ex). The desktop app is a client only — it does not bundle the server.

### Build dependencies

| Platform | Required packages |
|----------|------------------|
| **macOS** | Xcode Command Line Tools |
| **Windows** | [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11), [Build Tools for VS](https://visualstudio.microsoft.com/visual-cpp-build-tools/) |
| **Linux (Debian/Ubuntu)** | `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf` |
| **Linux (Fedora)** | `webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel` |

- [Rust](https://rustup.rs/) stable (≥ 1.77)
- [Node.js](https://nodejs.org/) ≥ 20

## Getting started

```bash
# Clone
git clone https://github.com/DigitalTolk/ex-desktop.git
cd ex-desktop

# Install dependencies and launch in dev mode
# (starts Vite dev server + Tauri window pointing at localhost:8080)
make dev
```

On first launch the app will ask for your workspace URL (e.g. `https://chat.yourcompany.com`).

### Available make targets

| Command | Description |
|---------|-------------|
| `make setup` | Install frontend npm dependencies |
| `make dev` | Launch dev build (hot reload) |
| `make build` | Produce a production installer |
| `make check` | Run Rust clippy + frontend lint |

### Manual commands

```bash
# Frontend only
npm --prefix frontend install
npm --prefix frontend run dev

# Rust check
cargo check --manifest-path src-tauri/Cargo.toml

# Full Tauri dev
cargo tauri dev

# Full Tauri production build
cargo tauri build
```

## Project structure

```
ex-desktop/
├── frontend/          React + TypeScript UI (derived from ex/frontend)
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs     App entry point, plugin registration
│   │   ├── main.rs    Binary entry point
│   │   └── commands.rs Tauri commands exposed to the frontend
│   ├── capabilities/
│   │   └── default.json  Permission declarations
│   └── tauri.conf.json   App config (identifier, window, bundle)
├── docs/
│   ├── API.md         ex server REST + WebSocket API reference
│   ├── DESKTOP_PLAN.md  Phased development plan
│   └── STACK_ANALYSIS.md Stack comparison + project future
├── .github/
│   └── workflows/
│       ├── ci.yml     Build on push / PR (3 platforms)
│       └── release.yml Publish installers on tag
└── Makefile
```

## Development

### Connecting to the ex server

In dev mode the Vite server proxies `/api` and `/auth` to `http://localhost:8080`. Start the ex server locally with:

```bash
# In the ex repository
docker compose up
```

Or set the `VITE_SERVER_URL` environment variable to point to a remote server.

### Adding Tauri commands

Commands live in [src-tauri/src/commands.rs](src-tauri/src/commands.rs). Register new commands in the `invoke_handler!` macro in [src-tauri/src/lib.rs](src-tauri/src/lib.rs).

```rust
#[tauri::command]
pub fn my_command(arg: String) -> Result<String, String> {
    Ok(format!("got: {}", arg))
}
```

Call from TypeScript:
```typescript
import { invoke } from '@tauri-apps/api/core'
const result = await invoke<string>('my_command', { arg: 'hello' })
```

## Releasing

Releases are created automatically by pushing a semver tag:

```bash
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions builds macOS (`.dmg`), Windows (`.msi` + `.exe`), and Linux (`.AppImage` + `.deb` + `.rpm`) installers and creates a draft release. Edit the release notes and publish when ready.

For code signing see [docs/DESKTOP_PLAN.md](docs/DESKTOP_PLAN.md#phase-9--packaging--distribution-3-days).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md) for the vulnerability disclosure policy.

## License

MIT — see [LICENSE](LICENSE).
