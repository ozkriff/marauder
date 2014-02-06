# See LICENSE file for copyright and license details.

# RUSTC_FLAGS += -Z debug-info
# RUSTC_FLAGS += --opt-level 3
RUSTC_FLAGS += -L deps
RUSTC_FLAGS += --link-args=-lglfw

RUSTC = rustc ${RUSTC_FLAGS}

all: marauder

SRC = \
  camera.rs \
  color.rs \
  geom.rs \
  gl_helpers.rs \
  glfw_events.rs \
  main.rs \
  misc.rs \
  map.rs \
  mesh.rs \
  obj.rs \
  tile_picker.rs \
  visualizer.rs

marauder: Makefile ${SRC}
	${RUSTC} main.rs -o marauder

clean:
	rm -f marauder

# vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
