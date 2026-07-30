SHELL := /bin/sh
PYTHON ?= python3
NPM ?= npm
CARGO ?= cargo

.PHONY: all install sync-version bump-patch build debug dev frontend check test fmt lint clean distclean version set-version package package-win64

all: build

install:
	$(NPM) install

version:
	@cat VERSION

sync-version:
	$(PYTHON) scripts/version.py

bump-patch:
	$(PYTHON) scripts/version.py --bump-patch

# Builds preserve the explicitly selected application version.
build: sync-version
	$(NPM) run tauri build

debug: sync-version
	$(NPM) run build
	cd src-tauri && $(CARGO) build

# Development hot-reload does not increment BUILD; it is a running session, not a packaged build.
dev: sync-version
	$(NPM) run tauri dev

frontend: sync-version
	$(NPM) run build

check: sync-version
	$(NPM) run build
	cd src-tauri && $(CARGO) check --all-targets

test: sync-version
	cd vendor/atascii && $(CARGO) test --all-features
	cd src-tauri && $(CARGO) test

fmt:
	cd vendor/atascii && $(CARGO) fmt --all
	cd src-tauri && $(CARGO) fmt --all

lint: sync-version
	cd vendor/atascii && $(CARGO) clippy --all-targets --all-features -- -D warnings
	cd src-tauri && $(CARGO) clippy --all-targets -- -D warnings

# Usage: make set-version VERSION=1.1.0-0
set-version:
	@test -n "$(VERSION)" || (echo "VERSION is required, e.g. make set-version VERSION=1.1.0-0" && exit 2)
	$(PYTHON) scripts/version.py --set "$(VERSION)"

package: build
	$(PYTHON) scripts/package_win64.py

package-win64:
	$(PYTHON) scripts/package_win64.py

clean:
	rm -rf dist src-tauri/target

distclean: clean
	rm -rf node_modules package-lock.json
