#version 450

layout(location = 0) in vec4 v_colour[];
layout(location = 1) in vec3 v_position[];
layout(location = 2) in vec3 v_eyePos[];
layout(location = 3) in vec3 v_normal[];

layout(location = 0) out vec4 e_colour[];
layout(location = 1) out vec3 e_position[];
layout(location = 2) out vec3 e_eyePos[];
layout(location = 3) out vec3 e_normal[];

layout (vertices = 3) out;

void main(void){
    // Passthroughs...
    e_colour[gl_InvocationID] = v_colour[gl_InvocationID];
    e_position[gl_InvocationID] = v_position[gl_InvocationID];
    e_eyePos[gl_InvocationID] = v_eyePos[gl_InvocationID];
    e_normal[gl_InvocationID] = v_normal[gl_InvocationID];

    gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;

    gl_TessLevelInner[0] = 1;
    gl_TessLevelOuter[0] = 1;
    gl_TessLevelOuter[1] = 1;
    gl_TessLevelOuter[2] = 1;
}
