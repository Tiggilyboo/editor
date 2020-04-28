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

layout(location = 0) out vec4 v_colour;
layout(location = 1) out vec3 v_position;
layout(location = 2) out vec3 v_eyePos;
layout(location = 3) out vec3 v_normal;

out gl_PerVertex {
  vec4 gl_Position;
};

void main() {
    v_eyePos = ubo.eye_position;
    v_colour = colour;
    v_position = vec3(ubo.model * vec4(position, 1.0));
    v_normal = vec3(0.0, 0.0, 1.0);
    
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(position, 1.0);
}
