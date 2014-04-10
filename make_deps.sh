# See LICENSE file for copyright and license details.
#!/bin/sh
set -e

get_glfw() {
    echo === glfw ===
    git clone --depth=1 https://github.com/glfw/glfw.git
    cd glfw
    cmake -DBUILD_SHARED_LIBS=ON
    make glfw
    mv src/libglfw* ..
    cd ..
}

get_glfw_rs() {
    echo === glfw-rs ===
    git clone --depth=1 https://github.com/bjz/glfw-rs
    cd glfw-rs
    get_glfw
    PKG_CONFIG_PATH=glfw/src make lib
    mv lib/libglfw* libglfw* ..
    cd ..
}

get_gl_rs() {
    echo === gl-rs ===
    git clone --depth=1 https://github.com/bjz/gl-rs
    cd gl-rs
    git submodule init
    git submodule update # TODO: two glfw-rs repos? :(
    make lib
    mv lib/libgl* ..
    cd ..
}

get_cgmath_rs() {
    echo === cgmath-rs ===
    git clone --depth=1 https://github.com/bjz/cgmath-rs
    rustc cgmath-rs/src/cgmath/lib.rs --out-dir .
}

get_stb_image() {
    echo === rust-stb-image ===
    git clone --depth=1 https://github.com/mozilla-servo/rust-stb-image
    cd rust-stb-image
    ./configure
    make
    mv libstb* ..
    cd ..
}

get_stb_tt_rs() {
    echo === stb-tt-rs ===
    git clone --depth=1 https://github.com/ozkriff/stb-tt-rs
    cd stb-tt-rs
    make
    mv libstb* ..
    cd ..
}

get_deps() {
    echo Creating 'deps' dir...
    mkdir deps
    cd deps
    get_glfw_rs
    get_gl_rs
    get_cgmath_rs
    get_stb_image
    get_stb_tt_rs
    cd ..
}

get_deps
echo Done!

# vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
