.PHONY: setup dev build icons lint test coverage check

RUST_COV_ENV :=
ifneq ($(wildcard /opt/homebrew/opt/llvm/bin/llvm-cov),)
RUST_COV_ENV := LLVM_COV=/opt/homebrew/opt/llvm/bin/llvm-cov LLVM_PROFDATA=/opt/homebrew/opt/llvm/bin/llvm-profdata
endif

setup:
	npm --prefix frontend install

dev: setup
	cargo tauri dev

build: setup
	cargo tauri build

icons:
	swift scripts/generate-icons.swift
	rm -rf /private/tmp/ex-app.iconset
	mkdir -p /private/tmp/ex-app.iconset
	sips -z 16 16 src-tauri/icons/app-icon.png --out /private/tmp/ex-app.iconset/icon_16x16.png >/dev/null
	sips -z 32 32 src-tauri/icons/app-icon.png --out /private/tmp/ex-app.iconset/icon_16x16@2x.png >/dev/null
	sips -z 32 32 src-tauri/icons/app-icon.png --out /private/tmp/ex-app.iconset/icon_32x32.png >/dev/null
	sips -z 64 64 src-tauri/icons/app-icon.png --out /private/tmp/ex-app.iconset/icon_32x32@2x.png >/dev/null
	sips -z 128 128 src-tauri/icons/app-icon.png --out /private/tmp/ex-app.iconset/icon_128x128.png >/dev/null
	sips -z 256 256 src-tauri/icons/app-icon.png --out /private/tmp/ex-app.iconset/icon_128x128@2x.png >/dev/null
	sips -z 256 256 src-tauri/icons/app-icon.png --out /private/tmp/ex-app.iconset/icon_256x256.png >/dev/null
	sips -z 512 512 src-tauri/icons/app-icon.png --out /private/tmp/ex-app.iconset/icon_256x256@2x.png >/dev/null
	sips -z 512 512 src-tauri/icons/app-icon.png --out /private/tmp/ex-app.iconset/icon_512x512.png >/dev/null
	cp src-tauri/icons/app-icon.png /private/tmp/ex-app.iconset/icon_512x512@2x.png
	iconutil -c icns -o src-tauri/icons/icon.icns /private/tmp/ex-app.iconset

lint:
	cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
	npm --prefix frontend run lint

test:
	npm --prefix frontend run test:coverage
	cd src-tauri && $(RUST_COV_ENV) cargo llvm-cov --workspace --fail-under-lines 90 --ignore-filename-regex 'src/(commands|lib|main)\.rs'
	mkdir -p src-tauri/coverage
	cd src-tauri && $(RUST_COV_ENV) cargo llvm-cov report --lcov --ignore-filename-regex 'src/(commands|lib|main)\.rs' --output-path coverage/lcov.info

coverage: test

check: lint test
