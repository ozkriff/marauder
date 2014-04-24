// See LICENSE file for copyright and license details.

#version 120

attribute vec3 in_vertex_coordinates;
attribute vec2 in_texture_coordinates;
uniform mat4 mvp_mat;
varying vec2 texture_coordinates;

void main() {
    texture_coordinates = in_texture_coordinates;
    vec4 v = vec4(in_vertex_coordinates, 1);
    gl_Position = mvp_mat * v;
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
