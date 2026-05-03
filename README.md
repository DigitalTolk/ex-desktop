# ex Desktop

Native desktop wrapper for [ex](https://github.com/DigitalTolk/ex) — a self-hosted team chat application.

Built with **Tauri v2** (Rust) and a small local bootstrap UI. The desktop app stores a workspace URL, then loads the server-hosted ex web interface inside the main webview.

## Features

- Mattermost-style wrapper around the existing ex web UI
- Local first-run setup screen for selecting a workspace URL
- Multi-server support (switch between self-hosted ex instances)
- System tray, global shortcut, auto-updater, and window-state persistence
- Narrow native bridge by default: the remote server UI is not granted local Tauri permissions

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
# (starts the local bootstrap UI + Tauri shell)
make dev
```

On first launch the app asks for your workspace URL (for example `https://chat.yourcompany.com`) and then hands the main window off to that server-hosted UI.

### Available make targets

| Command | Description |
|---------|-------------|
| `make setup` | Install frontend npm dependencies |
| `make dev` | Launch dev build (hot reload) |
| `make build` | Produce a production installer |
| `make check` | Run Rust clippy + frontend lint |

### Manual commands

```bash
# Local bootstrap UI only
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
├── frontend/          Local bootstrap/setup UI used before loading the remote server
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs     App entry point, tray, startup handoff to remote UI
│   │   ├── main.rs    Binary entry point
│   │   └── commands.rs Small local command surface for setup + window handoff
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

Start the ex server locally with:

```bash
# In the ex repository
docker compose up
```

Then launch `make dev`, enter the server URL into the local setup screen, and the shell will navigate the main webview to that remote UI.

### Local Commands

Commands live in [src-tauri/src/commands.rs](src-tauri/src/commands.rs). They are intentionally limited to local setup tasks such as storing the selected server URL and telling the main window to load it.

```rust
#[tauri::command]
pub fn save_server_url_and_load(app: AppHandle, url: String) -> Result<String, String> {
    // validate, persist, and navigate the main window
    Ok(url)
}
```

Call from TypeScript:
```typescript
import { invoke } from '@tauri-apps/api/core'
const result = await invoke<string>('save_server_url_and_load', { url: 'https://chat.example.com' })
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
