# See LICENSE file for copyright and license details.

all: build

.PHONY: build
build:
	cargo build

run: build
	RUST_BACKTRACE=1 ./target/marauder

# vim: set tabstop=4 shiftwidth=4 softtabstop=4:
