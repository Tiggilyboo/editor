#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UniformBufferObject {
  mat4 model;
  mat4 view;
  mat4 proj;
  vec3 eye_position;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 colour;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec3 fragPos;
layout(location = 2) out vec3 eyePos;
layout(location = 3) out vec3 normal;

out gl_PerVertex {
  vec4 gl_Position;
};

void main() {
    eyePos = ubo.eye_position;
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(position, 1.0);
    fragColor = colour;

    fragPos = vec3(ubo.model * vec4(position, 1.0));

    normal = vec3(0.0, 0.0, 1.0);
}
