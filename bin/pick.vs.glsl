// See LICENSE file for copyright and license details.

#version 120

attribute vec3 in_vertex_coordinates;
attribute vec3 color;
varying vec3 pass_color;
uniform mat4 mvp_mat;

void main() {
    vec4 v = vec4(in_vertex_coordinates, 1);
    gl_Position = mvp_mat * v;
    pass_color = color;
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
