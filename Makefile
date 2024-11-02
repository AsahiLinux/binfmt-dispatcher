DESTDIR ?=
PREFIX ?= /usr
BINDIR ?= $(PREFIX)/bin
BINFMTDIR ?= $(PREFIX)/lib/binfmt.d

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

uninstall: uninstall-bin uninstall-data

uninstall-bin:
	rm -f $(DESTDIR)$(BINDIR)/binfmt-dispatcher

uninstall-data:
	rm -f $(DESTDIR)$(BINFMTDIR)/binfmt-dispatcher-x86.conf
	rm -f $(DESTDIR)$(BINFMTDIR)/binfmt-dispatcher-x86_64.conf

.PHONY: check install install-bin install-data uninstall uninstall-bin uninstall-data
