# See LICENSE file for copyright and license details.

# RUSTC_FLAGS += -Z debug-info
# RUSTC_FLAGS += --opt-level 3
RUSTC_FLAGS += -L ~/rust_libs
RUSTC_FLAGS += -L .

all: marauder

misc.o: misc.rs
	rustc --lib misc.rs ${RUSTC_FLAGS}

win.o: win.rs
	rustc --lib win.rs ${RUSTC_FLAGS}

marauder: main.rs win.o misc.o
	rustc main.rs -o marauder ${RUSTC_FLAGS}

clean:
	rm -f marauder lib*-*.so
