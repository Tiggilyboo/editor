#version 450

layout(set = 0, binding = 1) uniform TextTransform {
  mat4 transform;
} ubo;

layout(location = 0) in vec2 position;
layout(location = 2) in vec2 tex_position;
layout(location = 4) in vec4 colour;

layout(location = 0) out vec2 f_tex_pos;
layout(location = 1) out vec4 f_colour;

void main() {
    f_colour = colour;
    f_tex_pos = tex_position;
    gl_Position = ubo.transform * vec4(position, 0.0, 1.0);
}
