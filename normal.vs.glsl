#version 130

in vec3 position;
uniform mat4 mvp_mat;

void main() {
  vec4 v = vec4(position, 1);
  gl_Position = mvp_mat * v;
}
