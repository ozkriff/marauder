// See LICENSE file for copyright and license details.

#version 120

uniform sampler2D basic_texture;
uniform vec4 basic_color;
varying vec2 texture_coordinates;

void main() {
    gl_FragColor = basic_color * texture2D(basic_texture, texture_coordinates);
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
