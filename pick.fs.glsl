// See LICENSE file for copyright and license details.

#version 130

in vec3 pass_color;
out vec4 out_color;

void main() {
    out_color = vec4(pass_color, 1.0);
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
