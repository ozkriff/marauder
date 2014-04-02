# See LICENSE file for copyright and license details.
#!/bin/sh
set -e

get_glfw() {
    echo === glfw ===
    git clone --depth=1 https://github.com/glfw/glfw.git
    cd glfw
    cmake -DBUILD_SHARED_LIBS=ON
    make glfw
    mv src/lib*.so* ..
    cd ..
}

get_glfw_rs() {
    echo === glfw-rs ===
    git clone --depth=1 https://github.com/bjz/glfw-rs
    cd glfw-rs
    get_glfw
    PKG_CONFIG_PATH=glfw/src make lib
    mv lib/*.rlib lib/*.so *.so* ..
    cd ..
}

get_gl_rs() {
    echo === gl-rs ===
    git clone --depth=1 https://github.com/bjz/gl-rs
    rustc gl-rs/src/gl/lib.rs --out-dir .
}

get_cgmath_rs() {
    echo === cgmath-rs ===
    git clone --depth=1 https://github.com/bjz/cgmath-rs
    rustc cgmath-rs/src/cgmath/lib.rs --out-dir .
}

get_stb_image() {
    echo === rust-stb-image ===
    # TODO: ozkriff -> mozilla-servo
    git clone --depth=1 https://github.com/ozkriff/rust-stb-image
    cd rust-stb-image
    ./configure
    make
    mv *.rlib *.a ..
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
    cd ..
}

get_deps
echo Done!

# vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
