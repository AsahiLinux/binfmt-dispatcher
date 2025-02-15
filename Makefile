DESTDIR ?=
PREFIX ?= /usr
BINDIR ?= $(PREFIX)/bin
BINFMTDIR ?= $(PREFIX)/lib/binfmt.d
DATADIR ?= $(PREFIX)/share
POLKITACTIONSDIR ?= $(DATADIR)/polkit-1/actions
SYSCONFIGDIR ?= /etc

RUSTFLAGS ?= --release

ROOTDIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

all: build

build:
	cargo build $(RUSTFLAGS)

check:
	cargo test $(RUSTFLAGS)

clean:
	rm -rf target

install: install-bin install-data

install-bin:
	install -Dpm0755 -t $(DESTDIR)$(BINDIR)/ target/release/binfmt-dispatcher

install-data:
	install -Dpm0644 -t $(DESTDIR)$(BINFMTDIR)/ data/binfmt-dispatcher-x86.conf
	install -Dpm0644 -t $(DESTDIR)$(BINFMTDIR)/ data/binfmt-dispatcher-x86_64.conf
	install -Dpm0644 -t $(DESTDIR)$(SYSCONFIGDIR)/ data/binfmt-dispatcher.toml
	install -Dpm0644 -t $(DESTDIR)$(POLKITACTIONSDIR)/ data/org.AsahiLinux.BinfmtDispatcher.policy

uninstall: uninstall-bin uninstall-data

uninstall-bin:
	rm -f $(DESTDIR)$(BINDIR)/binfmt-dispatcher

uninstall-data:
	rm -f $(DESTDIR)$(BINFMTDIR)/binfmt-dispatcher-x86.conf
	rm -f $(DESTDIR)$(BINFMTDIR)/binfmt-dispatcher-x86_64.conf
	rm -f $(DESTDIR)$(SYSCONFIGDIR)/binfmt-dispatcher.toml
	rm -f $(DESTDIR)$(POLKITACTIONSDIR)/org.AsahiLinux.BinfmtDispatcher.policy

.PHONY: check install install-bin install-data uninstall uninstall-bin uninstall-data
