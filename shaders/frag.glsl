#version 450

layout(location = 0) in vec4 colour;
layout(location = 1) in vec3 position;
layout(location = 2) in vec3 eyePos;
layout(location = 3) in vec3 normal;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 norm = normalize(normal);
    vec3 lightDir = normalize(eyePos - position);
    float diff = max(dot(norm, lightDir), 0.0);
    vec4 diffuse = vec4(diff * vec3(1.0, 1.0, 1.0), 1.0);

    vec4 result = 2.0 * diffuse * colour;

    outColor = result;
}
