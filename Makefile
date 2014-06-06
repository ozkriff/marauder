# See LICENSE file for copyright and license details.

# RUSTC_FLAGS += -g
# RUSTC_FLAGS += --opt-level 3
RUSTC_FLAGS += -L deps

RUSTC = rustc ${RUSTC_FLAGS}

SRC = \
	src/main.rs \
	src/core/mod.rs \
	src/core/misc.rs \
	src/core/conf.rs \
	src/core/map.rs \
	src/core/dir.rs \
	src/core/core.rs \
	src/core/game_state.rs \
	src/core/pathfinder.rs \
	src/core/types.rs \
	src/core/fs.rs \
	src/visualizer/mod.rs \
	src/visualizer/camera.rs \
	src/visualizer/geom.rs \
	src/visualizer/mgl.rs \
	src/visualizer/mesh.rs \
	src/visualizer/shader.rs \
	src/visualizer/texture.rs \
	src/visualizer/event_visualizer.rs \
	src/visualizer/obj.rs \
	src/visualizer/picker.rs \
	src/visualizer/types.rs \
	src/visualizer/visualizer.rs \
	src/visualizer/font_stash.rs \
	src/visualizer/gui.rs \
	src/visualizer/selection.rs \
	src/visualizer/context.rs \

all: bin/marauder

bin/marauder: Makefile ${SRC}
	${RUSTC} src/main.rs -o bin/marauder

run: bin/marauder
	RUST_BACKTRACE=1 ./bin/marauder

clean:
	rm -f bin/marauder

# vim: set tabstop=4 shiftwidth=4 softtabstop=4:
