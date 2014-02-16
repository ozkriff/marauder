# See LICENSE file for copyright and license details.
#!/bin/sh

echo Creating 'deps' dir...
mkdir deps
cd deps

# glfw
echo Downloading glfw...
git clone --depth=1 https://github.com/glfw/glfw.git
echo Building glfw...
cd glfw
cmake -DBUILD_SHARED_LIBS=ON
make glfw # build without examples or tests
cp src/libglfw.so* .. # copy dynamic libraries to 'deps' dir
cd ..

# glfw-rs
echo Downloading glfw-rs...
git clone --depth=1 https://github.com/bjz/glfw-rs
echo Building glfw-rs...
rustc glfw-rs/src/lib/lib.rs --out-dir .

# gl-rs
echo Downloading gl-rs...
git clone --depth=1 https://github.com/bjz/gl-rs
echo Building gl-rs...
rustc gl-rs/src/gl/lib.rs --out-dir .

# cgmath-rs
echo Downloading cgmath-rs...
git clone --depth=1 https://github.com/bjz/cgmath-rs
echo Building cgmath-rs...
rustc cgmath-rs/src/cgmath/lib.rs --out-dir .

# rust-stb-image
echo Downloading rust-stb-image...
git clone --depth=1 https://github.com/mozilla-servo/rust-stb-image
echo Building rust-stb-image...
cd rust-stb-image
./configure
make
cp *.rlib
cd ..

# Return from 'deps' dir
cd ..

echo Done!
