.SUFFIXES:

RUSTFLAGS ?= "-C target-cpu=native"

CARGO ?= RUSTFLAGS=$(RUSTFLAGS) cargo

sources = $(wildcard src/*.rs)

bin = nvfancontrol
debug_bin = target/debug/$(bin)
release_bin = target/release/$(bin)

.PHONY: build
build: $(debug_bin)

.PHONY: test
test: build
	$(CARGO) test

.PHONY: release-build
release-build: $(release_bin)

$(debug_bin): $(sources)
	$(CARGO) build

$(release_bin): $(sources)
	$(CARGO) build --release

.PHONY: clean
clean:
	cargo clean

.PHONY: clobber
clobber:
	cargo clean

.PHONY: install
install: $(release_bin)
	install -m 0775 $(release_bin) $(HOME)/.local/bin/
