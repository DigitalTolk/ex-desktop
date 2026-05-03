.PHONY: setup dev build lint test coverage check

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
