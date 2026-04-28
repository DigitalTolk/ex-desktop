.PHONY: setup dev build check

setup:
	npm --prefix frontend install

dev: setup
	cargo tauri dev

build: setup
	cargo tauri build

check:
	cd src-tauri && cargo clippy -- -D warnings
	npm --prefix frontend run lint
