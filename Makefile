# Makefile for pisi

PREFIX ?= /usr
BINDIR ?= $(PREFIX)/bin
MANDIR ?= $(PREFIX)/share/man/man1
COMPLETIONDIR ?= $(PREFIX)/share/bash-completion/completions
ZSHCOMPDIR ?= $(PREFIX)/share/zsh/site-functions
FISHCOMPDIR ?= $(PREFIX)/share/fish/vendor_completions.d
CARGO ?= cargo

TARGET = target/release/pisi
TARGET_LSPISI = target/release/lspisi
TARGET_UNPISI = target/release/unpisi

.PHONY: all build install clean uninstall man completions docs gen_man

all: build

build:
	$(CARGO) build --release

install: build
	install -Dm755 $(TARGET) $(DESTDIR)$(BINDIR)/pisi
	install -Dm755 $(TARGET_LSPISI) $(DESTDIR)$(BINDIR)/lspisi
	install -Dm755 $(TARGET_UNPISI) $(DESTDIR)$(BINDIR)/unpisi
	install -Dm644 mirrors.conf $(DESTDIR)/etc/pisi/mirrors.conf
	install -Dm644 pisi.conf $(DESTDIR)/etc/pisi/pisi.conf

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/pisi
	rm -f $(DESTDIR)$(BINDIR)/lspisi
	rm -f $(DESTDIR)$(BINDIR)/unpisi
	rm -f $(DESTDIR)/etc/pisi/mirrors.conf
	rm -f $(DESTDIR)/etc/pisi/pisi.conf

clean:
	$(CARGO) clean

man:
	@mkdir -p man
	@echo "Generating man page..."
	@$(CARGO) run --manifest-path pisi/Cargo.toml --features generate --bin gen_man 2>/dev/null || \
		( \
			echo "Building gen_man binary..."; \
			$(CARGO) build --manifest-path pisi/Cargo.toml --features generate --bin gen_man; \
			./target/debug/gen_man; \
		)
	@install -Dm644 man/pisi.1 $(DESTDIR)$(MANDIR)/pisi.1

completions:
	@mkdir -p man
	@$(CARGO) build --manifest-path pisi/Cargo.toml --features generate --bin gen_man
	@./target/debug/gen_man
	@install -Dm644 man/pisi.bash $(DESTDIR)$(COMPLETIONDIR)/pisi
	@install -Dm644 man/pisi.zsh $(DESTDIR)$(ZSHCOMPDIR)/_pisi
	@install -Dm644 man/pisi.fish $(DESTDIR)$(FISHCOMPDIR)/pisi.fish

docs: man completions

gen_man:
	@mkdir -p man
	@$(CARGO) build --manifest-path pisi/Cargo.toml --features generate --bin gen_man
	@./target/debug/gen_man
