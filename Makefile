# See LICENSE file for copyright and license details.

# RUSTC_FLAGS += -Z debug-info
# RUSTC_FLAGS += --opt-level 3
RUSTC_FLAGS += -W unnecessary-typecast
RUSTC_FLAGS += -W unnecessary-qualification
RUSTC_FLAGS += -W non-uppercase-statics
RUSTC_FLAGS += -W non-camel-case-types
RUSTC_FLAGS += -L ~/rust_libs
RUSTC_FLAGS += -L .

RUSTC = rustc ${RUSTC_FLAGS}

all: marauder

marauder: main.rs win.rs misc.rs
	${RUSTC} --lib misc.rs
	${RUSTC} --lib win.rs
	${RUSTC} main.rs -o marauder

clean:
	rm -f marauder lib*-*.so
