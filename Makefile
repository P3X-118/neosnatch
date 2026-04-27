PREFIX ?= /usr/local
DESTDIR ?=
TARGET ?= x86_64-unknown-linux-musl

.PHONY: build release install uninstall clean fmt check

build:
	cargo build

release:
	cargo build --release --target $(TARGET)

# Glibc release build (no toolchain prerequisite). Use this if you don't
# have the musl target installed: `rustup target add x86_64-unknown-linux-musl`.
release-glibc:
	cargo build --release

install: release
	install -Dm755 target/$(TARGET)/release/neosnatch $(DESTDIR)$(PREFIX)/bin/neosnatch
	install -Dm644 contrib/neosnatch.sh $(DESTDIR)/etc/profile.d/neosnatch.sh
	install -Dm644 contrib/config.example.toml $(DESTDIR)/etc/neosnatch/config.toml.example

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/neosnatch
	rm -f $(DESTDIR)/etc/profile.d/neosnatch.sh

fmt:
	cargo fmt

check:
	cargo check
	cargo clippy -- -D warnings

clean:
	cargo clean
