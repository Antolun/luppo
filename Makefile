# Makefile for luppo

PREFIX ?= /usr
BINDIR ?= $(PREFIX)/bin
MANDIR ?= $(PREFIX)/share/man/man1
COMPLETIONDIR ?= $(PREFIX)/share/bash-completion/completions
ZSHCOMPDIR ?= $(PREFIX)/share/zsh/site-functions
FISHCOMPDIR ?= $(PREFIX)/share/fish/vendor_completions.d
CARGO ?= cargo

TARGET = target/release/luppo
TARGET_LSLUPPO = target/release/lsluppo
TARGET_UNLUPPO = target/release/unluppo

.PHONY: all build install clean uninstall man completions docs gen_man

all: build

build:
	$(CARGO) build --release

install: build
	install -Dm755 $(TARGET) $(DESTDIR)$(BINDIR)/luppo
	install -Dm755 $(TARGET_LSLUPPO) $(DESTDIR)$(BINDIR)/lsluppo
	install -Dm755 $(TARGET_UNLUPPO) $(DESTDIR)$(BINDIR)/unluppo
	install -Dm644 mirrors.conf $(DESTDIR)/etc/luppo/mirrors.conf
	install -Dm644 luppo.conf $(DESTDIR)/etc/luppo/luppo.conf

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/luppo
	rm -f $(DESTDIR)$(BINDIR)/lsluppo
	rm -f $(DESTDIR)$(BINDIR)/unluppo
	rm -f $(DESTDIR)/etc/luppo/mirrors.conf
	rm -f $(DESTDIR)/etc/luppo/luppo.conf

clean:
	$(CARGO) clean

man:
	@mkdir -p man
	@echo "Generating man page..."
	@$(CARGO) run --manifest-path luppo/Cargo.toml --features generate --bin gen_man 2>/dev/null || \
		( \
			echo "Building gen_man binary..."; \
			$(CARGO) build --manifest-path luppo/Cargo.toml --features generate --bin gen_man; \
			./target/debug/gen_man; \
		)
	@install -Dm644 man/luppo.1 $(DESTDIR)$(MANDIR)/luppo.1

completions:
	@mkdir -p man
	@$(CARGO) build --manifest-path luppo/Cargo.toml --features generate --bin gen_man
	@./target/debug/gen_man
	@install -Dm644 man/luppo.bash $(DESTDIR)$(COMPLETIONDIR)/luppo
	@install -Dm644 man/luppo.zsh $(DESTDIR)$(ZSHCOMPDIR)/_luppo
	@install -Dm644 man/luppo.fish $(DESTDIR)$(FISHCOMPDIR)/luppo.fish

docs: man completions

gen_man:
	@mkdir -p man
	@$(CARGO) build --manifest-path luppo/Cargo.toml --features generate --bin gen_man
	@./target/debug/gen_man
