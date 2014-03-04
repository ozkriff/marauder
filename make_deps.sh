# See LICENSE file for copyright and license details.
#!/bin/sh

echo Creating 'deps' dir...
mkdir deps
cd deps

echo === glfw3 ===
git clone --depth=1 https://github.com/glfw/glfw.git
cd glfw
cmake -DBUILD_SHARED_LIBS=ON
make glfw # build without examples or tests
cp src/libglfw.so* .. # copy dynamic libraries to 'deps' dir
cd ..

echo === glfw-rs ===
git clone --depth=1 https://github.com/bjz/glfw-rs
rustc glfw-rs/src/lib/lib.rs --out-dir .

echo === gl-rs ===
git clone --depth=1 https://github.com/bjz/gl-rs
rustc gl-rs/src/gl/lib.rs --out-dir .

echo === cgmath-rs ===
git clone --depth=1 https://github.com/bjz/cgmath-rs
rustc cgmath-rs/src/cgmath/lib.rs --out-dir .

echo === rust-stb-image ===
git clone --depth=1 https://github.com/mozilla-servo/rust-stb-image
cd rust-stb-image
./configure
make
cp *.rlib *.a ..
cd ..

# Return from 'deps' dir
cd ..

echo Done!
