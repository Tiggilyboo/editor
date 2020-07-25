#version 450
#
layout(set = 0, binding = 0) uniform TextTransform {
  mat4 transform;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 colour;

layout(location = 0) out vec4 f_colour;

void main() {
    f_colour = colour;
    gl_Position = ubo.transform * vec4(position, 1.0);
}
