# See LICENSE file for copyright and license details.

// RUSTC_FLAGS += -Z debug-info
RUSTC_FLAGS += --link-args -lm
RUSTC_FLAGS += --opt-level 3
RUSTC_FLAGS += -L ~/lib 

all: marauder

marauder: main.rs
	rustc main.rs -o marauder ${RUSTC_FLAGS}

clean:
	rm -f marauder
