// See LICENSE file for copyright and license details.

#version 130

in vec3 in_vertex_coordinates;
in vec3 color;
out vec3 pass_color;
uniform mat4 mvp_mat;

void main() {
  vec4 v = vec4(in_vertex_coordinates, 1);
  gl_Position = mvp_mat * v;
  pass_color = color;
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
