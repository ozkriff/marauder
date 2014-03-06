# See LICENSE file for copyright and license details.
#!/bin/sh
set -e

echo Creating 'deps' dir...
mkdir deps
cd deps # '.' -> './deps'

echo === glfw-rs ===
git clone --depth=1 https://github.com/bjz/glfw-rs
cd glfw-rs
git clone --depth=1 https://github.com/glfw/glfw.git
cd glfw; cmake -DBUILD_SHARED_LIBS=ON; make glfw; cp src/lib*.so* ..; cd ..
PKG_CONFIG_PATH=glfw/src make lib
cp lib/*.rlib lib/*.so *.so* ..
cd ..

echo === gl-rs ===
git clone --depth=1 https://github.com/bjz/gl-rs
rustc gl-rs/src/gl/lib.rs --out-dir .

echo === cgmath-rs ===
git clone --depth=1 https://github.com/bjz/cgmath-rs
rustc cgmath-rs/src/cgmath/lib.rs --out-dir .

echo === rust-stb-image ===
git clone --depth=1 https://github.com/mozilla-servo/rust-stb-image
cd rust-stb-image; ./configure; make; cp *.rlib *.a ..; cd ..

cd .. # './deps' -> '.'

echo Done!
