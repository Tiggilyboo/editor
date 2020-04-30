#version 450

layout(location = 0) in vec4 v_colour;
layout(location = 1) in vec3 v_position;
layout(location = 2) in vec3 v_eyePos;
layout(location = 3) in vec3 v_normal;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 norm = normalize(v_normal);
    vec3 lightDir = normalize(v_eyePos + vec3(0.0, 0.0, 1.0) - v_position);
    float diff = max(dot(norm, lightDir), 0.0);
    vec4 diffuse = vec4(diff * vec3(1.0, 1.0, 1.0), 1.0);

    vec4 result = 1.1 * diffuse * v_colour;

    outColor = result;
}
