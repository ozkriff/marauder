# See LICENSE file for copyright and license details.

all: target/marauder

target/marauder:
	cargo build

run: target/marauder
	RUST_BACKTRACE=1 ./target/marauder

clean:
	rm -f target/marauder

# vim: set tabstop=4 shiftwidth=4 softtabstop=4:
