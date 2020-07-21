#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 colour;

layout(location = 0) out vec4 f_colour;

void main() {
    f_colour = colour;
    gl_Position = vec4(position, 1.0);
}
