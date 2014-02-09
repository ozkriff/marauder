// See LICENSE file for copyright and license details.

#version 130

in vec3 position;
in vec2 vt;
uniform mat4 mvp_mat;
out vec2 texture_coordinates;

void main() {
  texture_coordinates = vt;
  vec4 v = vec4(position, 1);
  gl_Position = mvp_mat * v;
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
