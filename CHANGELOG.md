# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial Tauri v2 project scaffold with Rust backend
- React frontend (derived from [ex](https://github.com/DigitalTolk/ex) server frontend)
- All Tauri v2 plugin dependencies wired in for planned phases:
  - `tauri-plugin-notification` (Phase 2)
  - `tauri-plugin-store` (Phase 4)
  - `tauri-plugin-updater` (Phase 5)
  - `tauri-plugin-dialog` / `tauri-plugin-fs` (Phase 6)
  - `tauri-plugin-window-state` (Phase 7)
  - `tauri-plugin-deep-link` (Phase 8)
  - `keyring` for OS keychain (Phase 1)
- GitHub Actions CI: build matrix for macOS, Windows, Linux on every push and PR
- GitHub Actions release workflow: publish signed installers on semver tag push
- Makefile with `setup`, `dev`, `build`, `check` targets
- API reference for the ex server REST + WebSocket API (`docs/API.md`)
- Phased development plan (`docs/DESKTOP_PLAN.md`)
- Stack analysis and project future analysis (`docs/STACK_ANALYSIS.md`)

[Unreleased]: https://github.com/DigitalTolk/ex-desktop/compare/HEAD...HEAD
