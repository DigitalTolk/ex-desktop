# Contributing to ex Desktop

Thank you for your interest in contributing!

## Before you start

- Search [existing issues](https://github.com/DigitalTolk/ex-desktop/issues) before opening a new one.
- For significant changes, open an issue to discuss the approach before sending a PR.
- All contributions are subject to the [MIT License](LICENSE).

## Development setup

```bash
git clone https://github.com/DigitalTolk/ex-desktop.git
cd ex-desktop
make dev
```

See [README.md](README.md) for system prerequisites per platform.

## Workflow

1. Fork the repository and create a branch from `main`:
   ```bash
   git checkout -b feat/my-feature
   ```

2. Make your changes. Keep commits focused — one logical change per commit.

3. Run the checks locally before pushing:
   ```bash
   make check
   ```

4. Open a pull request against `main`. Fill in the PR template.

## Code style

**Rust** — follow `rustfmt` defaults. Run `cargo fmt` before committing.

**TypeScript / React** — follow the existing ESLint config. Run `npm --prefix frontend run lint`.

**Comments** — only write a comment when the *why* is non-obvious. Avoid restating what the code already says.

## Commit messages

Use the imperative mood and keep the subject line under 72 characters:

```
Add OS keychain storage for auth tokens
Fix notification permission request on first launch
```

Reference issues where relevant: `Fixes #42`.

## Releases

Maintainers handle releases. A new release is triggered by pushing a semver tag (`v0.x.y`). Update `CHANGELOG.md` under `[Unreleased]` before tagging.

## Security vulnerabilities

Do **not** open a public issue for security bugs. See [SECURITY.md](SECURITY.md).
