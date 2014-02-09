#version 130

uniform sampler2D basic_texture;
in vec2 texture_coordinates;
out vec4 out_color;

void main() {
  out_color = texture(basic_texture, texture_coordinates);
}
