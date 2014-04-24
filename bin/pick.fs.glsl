// See LICENSE file for copyright and license details.

#version 120

varying vec3 pass_color;

void main() {
    gl_FragColor = vec4(pass_color, 1.0);
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
