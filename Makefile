# See LICENSE file for copyright and license details.

# RUSTC_FLAGS += -Z debug-info
# RUSTC_FLAGS += --opt-level 3
RUSTC_FLAGS += -L ~/lib 
RUSTC_FLAGS += -L .

all: marauder

win.o: win.rs
	rustc --lib win.rs ${RUSTC_FLAGS}

marauder: main.rs win.o
	rustc main.rs -o marauder ${RUSTC_FLAGS}

clean:
	rm -f marauder lib*-*.so
