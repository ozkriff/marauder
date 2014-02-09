// See LICENSE file for copyright and license details.

#version 130

uniform sampler2D basic_texture;
in vec2 texture_coordinates;
out vec4 out_color;

void main() {
  out_color = texture(basic_texture, texture_coordinates);
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
