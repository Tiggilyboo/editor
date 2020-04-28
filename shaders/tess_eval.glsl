#version 450
layout(location = 0) in vec4 e_colour[];
layout(location = 1) in vec3 e_position[];
layout(location = 2) in vec3 e_eyePos[];
layout(location = 3) in vec3 e_normal[];

layout(location = 0) out vec4 f_colour;
layout(location = 1) out vec3 f_position;
layout(location = 2) out vec3 f_eyePos;
layout(location = 3) out vec3 f_normal;

layout(triangles, equal_spacing, ccw) in;

void main(void)
{
    // Passthroughs...
    f_colour = e_colour[0];
    f_position = e_position[0];
    f_eyePos = e_eyePos[0];
    f_normal = e_normal[0];

    vec4 vert_x = gl_in[0].gl_Position;
    vec4 vert_y = gl_in[1].gl_Position;
    vec4 vert_z = gl_in[2].gl_Position;

    gl_Position = vec4(
        gl_TessCoord.x * vert_x.x + gl_TessCoord.y * vert_y.x + gl_TessCoord.z * vert_z.x,
        gl_TessCoord.x * vert_x.y + gl_TessCoord.y * vert_y.y + gl_TessCoord.z * vert_z.y,
        gl_TessCoord.x * vert_x.z + gl_TessCoord.y * vert_y.z + gl_TessCoord.z * vert_z.z,
        1.0
    );
}
