#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_position;
layout(location = 2) in vec4 colour;

layout(location = 0) out vec2 v_tex_position;
layout(location = 1) out vec4 v_colour;

void main() {
  gl_Position = vec4(position, 1.0, 1.0);
  v_tex_position = tex_position;
  v_colour = colour;
}

