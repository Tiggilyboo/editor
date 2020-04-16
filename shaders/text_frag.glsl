#version 450

layout(location = 0) in vec2 v_tex_position;
layout(location = 1) in vec4 v_colour;

layout(location = 0) out vec4 f_colour;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
  f_colour = v_colour * texture(tex, v_tex_position)[0];
}
