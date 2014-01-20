# See LICENSE file for copyright and license details.

# RUSTC_FLAGS += -Z debug-info
# RUSTC_FLAGS += --opt-level 3
RUSTC_FLAGS += -L ~/rust_libs
RUSTC_FLAGS += --link-args=-lglfw

RUSTC = rustc ${RUSTC_FLAGS}

all: marauder

marauder: main.rs win.rs misc.rs
	${RUSTC} main.rs -o marauder

clean:
	rm -f marauder
