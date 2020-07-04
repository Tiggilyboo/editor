#version 450

layout(set = 0, binding = 0) uniform sampler2D font_tex;

layout(location = 0) in vec2 f_tex_pos;
layout(location = 1) in vec4 f_colour;

layout(location = 0) out vec4 out_colour;

void main() {
    float alpha = texture(font_tex, f_tex_pos).r;
    if (alpha <= 0.0) {
      alpha = 1.0;
        //discard;
    }
    out_colour = f_colour * vec4(1.0, 1.0, 1.0, alpha);
}
