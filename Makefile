# See LICENSE file for copyright and license details.

# RUSTC_FLAGS += -g
# RUSTC_FLAGS += --opt-level 3
RUSTC_FLAGS += -L deps
RUSTC_FLAGS += -C link-args=-lglfw

RUSTC = rustc ${RUSTC_FLAGS}

all: marauder

SRC = \
  main.rs \
  core/mod.rs \
  core/misc.rs \
  core/map.rs \
  core/dir.rs \
  core/core.rs \
  core/game_state.rs \
  core/pathfinder.rs \
  core/core_types.rs \
  visualizer/mod.rs \
  visualizer/camera.rs \
  visualizer/geom.rs \
  visualizer/gl_helpers.rs \
  visualizer/mesh.rs \
  visualizer/shader.rs \
  visualizer/texture.rs \
  visualizer/event_visualizer.rs \
  visualizer/obj.rs \
  visualizer/tile_picker.rs \
  visualizer/gl_types.rs \
  visualizer/visualizer.rs \


marauder: Makefile ${SRC}
	${RUSTC} main.rs -o marauder

clean:
	rm -f marauder

# vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
