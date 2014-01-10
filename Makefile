# See LICENSE file for copyright and license details.

# RUSTC_FLAGS += -Z debug-info
# RUSTC_FLAGS += --opt-level 3
RUSTC_FLAGS += -L ~/lib 
RUSTC_FLAGS += -L .

all: marauder

main.o: main.rs
	rustc --lib main.rs ${RUSTC_FLAGS}

marauder: marauder.rs main.o
	rustc marauder.rs -o marauder ${RUSTC_FLAGS}

clean:
	rm -f marauder lib*-*.so
