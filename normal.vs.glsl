// See LICENSE file for copyright and license details.

#version 130

in vec3 in_vertex_coordinates;
in vec2 in_texture_coordinates;
uniform mat4 mvp_mat;
out vec2 texture_coordinates;

void main() {
  texture_coordinates = in_texture_coordinates;
  vec4 v = vec4(in_vertex_coordinates, 1);
  gl_Position = mvp_mat * v;
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
